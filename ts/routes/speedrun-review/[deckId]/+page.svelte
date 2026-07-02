<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";

    import ConceptLearn from "../ConceptLearn.svelte";
    import {
        answerCard,
        type Concept,
        findConcept,
        type LearningBlock,
        nextCard,
        type Node,
        openDeck,
        type Problem,
        type Rating,
        type ReviewCard as ReviewCardData,
        recordLearned,
        rotateProblem,
        scaffoldSteps,
        showDecks,
    } from "../lib";
    import NewTopicIntro from "../NewTopicIntro.svelte";
    import PracticeRecall from "../PracticeRecall.svelte";
    import ProblemCard from "../ProblemCard.svelte";
    import ScaffoldPicker from "../ScaffoldPicker.svelte";
    import UpgradeAnimation from "../UpgradeAnimation.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    // The active screen. Every study step becomes a `View` whose callback
    // resolves the promise the driver is awaiting, so the loop below reads as
    // straight-line async instead of a tangle of state flags.
    type View =
        | { kind: "loading" }
        | { kind: "error"; message: string }
        | { kind: "intro"; root: Node; topicNodeId: string; onStart: () => void }
        | {
              kind: "learn";
              concept: Concept;
              current: number;
              total: number;
              onNext: () => void;
          }
        | { kind: "practice"; concept: Concept; onRate: (rating: Rating) => void }
        | { kind: "scaffold"; root: Node; conceptId: string; onComplete: () => void }
        | {
              kind: "problem";
              concept: Concept;
              problem: Problem | null;
              state: string;
              onRate: (rating: Rating) => void;
          }
        | { kind: "upgrade"; from: string; to: string; onDone: () => void }
        | { kind: "done" };

    let view: View = data.hierarchy
        ? { kind: "loading" }
        : { kind: "error", message: data.error ?? "This deck could not be opened." };
    let leaving = false;

    const deckId = data.deckId;
    const root = data.hierarchy?.root ?? null;
    // Concepts already taught in an earlier session; used to skip re-teaching
    // within a resumed learning block.
    const seen = new Set<string>(
        Object.entries(data.progress ?? {})
            .filter(([, entry]) => entry.seen)
            .map(([id]) => id),
    );
    // Applying/Mastering rotate through a concept's problems, one per review.
    const rotation = new Map<string, number>();

    function step<T>(make: (resolve: (value: T) => void) => View): Promise<T> {
        return new Promise<T>((resolve) => {
            view = make(resolve);
        });
    }

    function conceptFor(conceptId: string): Concept {
        const found = root && findConcept(root, conceptId);
        // reconcile removes orphaned cards before next-card, so this is only a
        // guard: a missing concept still gets a gradable placeholder.
        return (
            found ?? {
                id: conceptId,
                title: "Untitled concept",
                content: "",
                problems: [],
            }
        );
    }

    function nextRotation(conceptId: string): number {
        const current = rotation.get(conceptId) ?? 0;
        rotation.set(conceptId, current + 1);
        return current;
    }

    async function grade(
        cardId: string,
        conceptId: string,
        rating: Rating,
    ): Promise<void> {
        const result = await answerCard(deckId, cardId, conceptId, rating);
        if (result.upgraded) {
            await step<void>((resolve) => ({
                kind: "upgrade",
                from: result.from,
                to: result.to,
                onDone: resolve,
            }));
        }
    }

    async function runReview(card: ReviewCardData): Promise<void> {
        const concept = conceptFor(card.conceptId);
        let rating: Rating;
        if (card.state === "practicing") {
            rating = await step<Rating>((resolve) => ({
                kind: "practice",
                concept,
                onRate: resolve,
            }));
        } else {
            // Applying scaffolds down the tree first; Mastering (and anything
            // unexpected) goes straight to the MCQ.
            if (
                card.state === "hierarchy" &&
                root &&
                scaffoldSteps(root, card.conceptId).length
            ) {
                await step<void>((resolve) => ({
                    kind: "scaffold",
                    root,
                    conceptId: card.conceptId,
                    onComplete: resolve,
                }));
            }
            const problem = rotateProblem(
                concept.problems,
                nextRotation(card.conceptId),
            );
            rating = await step<Rating>((resolve) => ({
                kind: "problem",
                concept,
                problem,
                state: card.state,
                onRate: resolve,
            }));
        }
        await grade(card.cardId, card.conceptId, rating);
    }

    async function runLearningBlock(block: LearningBlock): Promise<void> {
        if (!root) {
            return;
        }
        await step<void>((resolve) => ({
            kind: "intro",
            root,
            topicNodeId: block.topicNodeId,
            onStart: resolve,
        }));

        const pending = block.conceptIds.filter((id) => !seen.has(id));
        let upgraded = false;
        for (let i = 0; i < pending.length; i++) {
            const id = pending[i];
            await step<void>((resolve) => ({
                kind: "learn",
                concept: conceptFor(id),
                current: block.learnedCount + i + 1,
                total: block.totalCount,
                onNext: resolve,
            }));
            seen.add(id);
            const result = await recordLearned(deckId, [id]);
            if (result.upgraded) {
                upgraded = true;
                await step<void>((resolve) => ({
                    kind: "upgrade",
                    from: result.from,
                    to: result.to,
                    onDone: resolve,
                }));
            }
        }

        // A resumed block can arrive with everything already seen; force the
        // topic-gated flip so it never stalls.
        if (!upgraded) {
            const result = await recordLearned(deckId, block.conceptIds);
            if (result.upgraded) {
                await step<void>((resolve) => ({
                    kind: "upgrade",
                    from: result.from,
                    to: result.to,
                    onDone: resolve,
                }));
            }
        }
    }

    async function run(): Promise<void> {
        if (!root) {
            return;
        }
        try {
            for (;;) {
                view = { kind: "loading" };
                const next = await nextCard(deckId);
                if (next.kind === "done") {
                    view = { kind: "done" };
                    return;
                }
                if (next.kind === "learning_block") {
                    await runLearningBlock(next);
                } else {
                    await runReview(next);
                }
            }
        } catch (err) {
            view = {
                kind: "error",
                message: err instanceof Error ? err.message : String(err),
            };
        }
    }

    async function toOverview(): Promise<void> {
        leaving = true;
        try {
            await openDeck(deckId);
        } catch {
            leaving = false;
        }
    }

    async function toHome(): Promise<void> {
        leaving = true;
        try {
            await showDecks();
        } catch {
            leaving = false;
        }
    }

    // Whether the current view is a live study step (so the persistent exit and
    // Escape apply). The done/error views carry their own exits.
    $: inSession = view.kind !== "done" && view.kind !== "error";

    // Escape leaves the session; answered cards are already committed, so there
    // is nothing to lose and no confirm to nag through.
    function onKeydown(event: KeyboardEvent): void {
        if (event.key === "Escape" && inSession && !leaving) {
            event.preventDefault();
            toOverview();
        }
    }

    onMount(run);
</script>

<svelte:window on:keydown={onKeydown} />

<div class="stage">
    {#if inSession}
        <button
            class="exit"
            type="button"
            on:click={toOverview}
            disabled={leaving}
            title="End session and return to the deck (Esc)"
        >
            <span aria-hidden="true">←</span>
            End session
        </button>
    {/if}
    {#if view.kind === "loading"}
        <p class="muted" role="status">Loading</p>
    {:else if view.kind === "error"}
        <div class="message">
            <p class="message-title">Could not study this deck</p>
            <p class="muted">{view.message}</p>
            <div class="actions">
                <button class="btn" on:click={run}>Try again</button>
                <button class="btn" on:click={toHome} disabled={leaving}>
                    All decks
                </button>
            </div>
        </div>
    {:else if view.kind === "intro"}
        <NewTopicIntro
            root={view.root}
            topicNodeId={view.topicNodeId}
            onStart={view.onStart}
        />
    {:else if view.kind === "learn"}
        <ConceptLearn
            concept={view.concept}
            current={view.current}
            total={view.total}
            onNext={view.onNext}
        />
    {:else if view.kind === "practice"}
        <PracticeRecall concept={view.concept} onRate={view.onRate} />
    {:else if view.kind === "scaffold"}
        <ScaffoldPicker
            root={view.root}
            conceptId={view.conceptId}
            onComplete={view.onComplete}
        />
    {:else if view.kind === "problem"}
        <ProblemCard
            concept={view.concept}
            problem={view.problem}
            state={view.state}
            onRate={view.onRate}
        />
    {:else if view.kind === "upgrade"}
        <UpgradeAnimation from={view.from} to={view.to} onDone={view.onDone} />
    {:else if view.kind === "done"}
        <div class="message">
            <p class="eyebrow">Session complete</p>
            <p class="message-title">Nothing due right now</p>
            <p class="muted">You have cleared every card the scheduler had ready.</p>
            <div class="actions">
                <button class="btn primary" on:click={toOverview} disabled={leaving}>
                    Back to overview
                </button>
                <button class="btn" on:click={toHome} disabled={leaving}>
                    All decks
                </button>
            </div>
        </div>
    {/if}
</div>

<style lang="scss">
    @use "../sr-tokens" as srt;

    .stage {
        box-sizing: border-box;
        position: relative;
        min-height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 1rem;
        padding: 2.5rem 1.5rem 3rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;

        @include srt.sr-tokens;
    }

    // A quiet, always-available way out during a live session; the done/error
    // views carry their own exits, so it hides there.
    .exit {
        position: absolute;
        top: 1.1rem;
        left: 1.1rem;
        z-index: 1;
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.4rem 0.7rem;
        border: 1px solid var(--sr-line-2);
        border-radius: 7px;
        background: var(--sr-panel);
        color: var(--sr-ink-3);
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        cursor: pointer;
        transition:
            color 0.12s ease,
            border-color 0.12s ease,
            background 0.12s ease;
    }
    .exit:hover:not(:disabled) {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    .exit:disabled {
        opacity: 0.5;
        cursor: default;
    }

    .muted {
        margin: 0;
        color: var(--sr-ink-3);
        font-family: var(--sr-mono);
        font-size: 0.8rem;
        letter-spacing: 0.04em;
    }

    .message {
        max-width: 34rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        text-align: center;
    }
    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .eyebrow {
        color: var(--sr-signal);
    }
    .message-title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 700;
        letter-spacing: -0.015em;
    }
    .actions {
        margin-top: 1rem;
        display: flex;
        gap: 0.6rem;
        flex-wrap: wrap;
        justify-content: center;
    }
    .btn {
        appearance: none;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        padding: 0.6rem 1.3rem;
        background: var(--sr-panel);
        color: var(--sr-ink);
        font: inherit;
        font-weight: 560;
        cursor: pointer;
    }
    .btn:hover:not(:disabled) {
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    .btn:disabled {
        opacity: 0.5;
        cursor: default;
    }
    .btn.primary {
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
    }
    .btn.primary:hover:not(:disabled) {
        filter: brightness(1.04);
    }

    .stage :global(:focus-visible) {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 6px;
    }
</style>
