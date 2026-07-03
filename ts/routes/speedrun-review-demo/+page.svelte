<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Developer gallery of the bespoke Speedrun screens. It drives the *real* screen
components — the decks home, the study overview, the deck builder, and every
study-session sub-state — with the inline mock data in demo-fixtures, so every
screen and state can be walked without a collection.

Nothing here touches the backend. The reused components are the presentational
ones the real routes render (DecksView, StudyOverview, HierarchyEditor, and the
session cards); the builder runs with persist=false against a private cloned
hierarchy, so no speedrun* RPC ever fires and no deck is created, saved, or
deleted.
-->
<script lang="ts">
    import DecksView from "../speedrun-decks/DecksView.svelte";
    import ConceptModal from "../speedrun-hierarchy/ConceptModal.svelte";
    import HierarchyEditor from "../speedrun-hierarchy/HierarchyEditor.svelte";
    import ConceptLearn from "../speedrun-review/ConceptLearn.svelte";
    import GuidedCard from "../speedrun-review/GuidedCard.svelte";
    import {
        type AnswerResult,
        type Concept,
        findConcept,
        type Hierarchy,
        type Rating,
        rotateProblem,
    } from "../speedrun-review/lib";
    import NewTopicIntro from "../speedrun-review/NewTopicIntro.svelte";
    import PracticeRecall from "../speedrun-review/PracticeRecall.svelte";
    import ProblemCard from "../speedrun-review/ProblemCard.svelte";
    import TopicLearned from "../speedrun-review/TopicLearned.svelte";
    import UpgradeAnimation from "../speedrun-review/UpgradeAnimation.svelte";
    import StudyOverview from "../speedrun-study/StudyOverview.svelte";
    import {
        cloneHierarchy,
        DEMO_CONCEPT_ID,
        DEMO_DECKS,
        DEMO_HIERARCHY,
        DEMO_LEARNING_CONCEPTS,
        DEMO_STUDY,
        DEMO_STUDY_NOTSTARTED,
        DEMO_TOPIC_NODE_ID,
    } from "./demo-fixtures";

    type SceneId =
        | "decks"
        | "decks-empty"
        | "builder"
        | "concept"
        | "study"
        | "study-empty"
        | "intro"
        | "learn"
        | "topic-learned"
        | "practice"
        | "levelup-pg"
        | "guided"
        | "levelup-gs"
        | "solo"
        | "done";

    interface Scene {
        id: SceneId;
        label: string;
        caption: string;
    }

    // Ordered as the real journey reads top to bottom: browse decks, build one,
    // then study it screen by screen.
    const scenes: Scene[] = [
        {
            id: "decks",
            label: "Decks",
            caption: "The decks home. Each row shows completion and opens to study or edit.",
        },
        {
            id: "decks-empty",
            label: "Decks — empty",
            caption: "The home before any deck exists.",
        },
        {
            id: "builder",
            label: "Builder",
            caption: "Name the tree, add topics, open a leaf to write its concepts.",
        },
        {
            id: "concept",
            label: "Concept editor",
            caption: "A concept's title, description, and four-choice problems.",
        },
        {
            id: "study",
            label: "Study overview",
            caption: "Today's queue, the three scores, and the concept tree.",
        },
        {
            id: "study-empty",
            label: "Study — not started",
            caption: "A deck not started yet: nothing due, every score awaiting data.",
        },
        {
            id: "intro",
            label: "New topic",
            caption: "New topic reveal. The path lights down the tree to the leaf.",
        },
        {
            id: "learn",
            label: "Learn",
            caption:
                "Two contrasting cases, note the shared idea, then reveal the concept.",
        },
        {
            id: "topic-learned",
            label: "Topic learned",
            caption: "Every concept clears Learn together and advances to Practice.",
        },
        {
            id: "practice",
            label: "Practice",
            caption: "Free-response recall, reveal, then rate the difficulty.",
        },
        {
            id: "levelup-pg",
            label: "Practice to Guided",
            caption: "Level up: the concept advances from Practice to Guided.",
        },
        {
            id: "guided",
            label: "Guided",
            caption: "Locate the concept in the tree, then answer the question.",
        },
        {
            id: "levelup-gs",
            label: "Guided to Solo",
            caption: "Level up: the concept advances from Guided to Solo.",
        },
        {
            id: "solo",
            label: "Solo",
            caption: "Answer the same question with no locating step.",
        },
        {
            id: "done",
            label: "Session complete",
            caption: "Nothing due. The finished session state.",
        },
    ];

    // The single concept the Practice/Guided/Solo scenes drill, so one idea is
    // seen climbing the ladder. The fixture guarantees it exists.
    const featured =
        findConcept(DEMO_HIERARCHY.root, DEMO_CONCEPT_ID) ?? DEMO_LEARNING_CONCEPTS[0];

    // The screens that paint their own full-bleed layout (own header, background,
    // and centering). The rest are cards that centre in the stage.
    const fullBleed = new Set<SceneId>([
        "decks",
        "decks-empty",
        "builder",
        "study",
        "study-empty",
    ]);
    const animated = new Set<SceneId>([
        "intro",
        "topic-learned",
        "levelup-pg",
        "levelup-gs",
    ]);
    const interactive = new Set<SceneId>([
        "builder",
        "concept",
        "learn",
        "practice",
        "guided",
        "solo",
    ]);

    let sceneIndex = 0;
    // Bumped on every scene entry and every replay/reset; used as the {#key} for
    // the stage so animated scenes replay, interactive cards reset, and the
    // builder/concept editors start from a pristine clone.
    let stageKey = 0;

    // Per-scene transient state.
    let learnIndex = 0;
    // Private clones the builder / concept editors mutate, rebuilt on every entry
    // so edits never leak into the shared study fixtures.
    let authoring: Hierarchy = cloneHierarchy();
    let conceptDraft: Concept | null = findConcept(
        cloneHierarchy().root,
        DEMO_CONCEPT_ID,
    );

    // The contextual toolbar action: replay an animation, reset an interactive
    // card/editor, or nothing on the static scenes.
    function actionFor(id: SceneId): string | null {
        if (animated.has(id)) {
            return "Replay";
        }
        if (interactive.has(id)) {
            return "Reset";
        }
        return null;
    }

    $: scene = scenes[sceneIndex];
    $: learnConcept = DEMO_LEARNING_CONCEPTS[learnIndex];
    // Guided and Solo drill the featured concept's first problem.
    $: featuredProblem = rotateProblem(featured.problems, 0);
    $: sceneAction = actionFor(scene.id);
    $: isFull = fullBleed.has(scene.id);

    function enter(index: number): void {
        sceneIndex = index;
        learnIndex = 0;
        // Rebuild the editor clones so every visit (and every Reset) starts from
        // the pristine fixture and never leaks edits into the study scenes.
        authoring = cloneHierarchy();
        conceptDraft = findConcept(cloneHierarchy().root, DEMO_CONCEPT_ID);
        stageKey += 1;
    }

    function goTo(id: SceneId): void {
        const index = scenes.findIndex((s) => s.id === id);
        if (index >= 0) {
            enter(index);
        }
    }

    function goPrev(): void {
        if (sceneIndex > 0) {
            enter(sceneIndex - 1);
        }
    }

    function goNext(): void {
        if (sceneIndex < scenes.length - 1) {
            enter(sceneIndex + 1);
        }
    }

    // Continue through the learning block: advance the concept counter, then hand
    // off to the topic-learned screen once the block is taught (the topic-gated
    // Learn -> Practice flip that upgrades every concept together).
    function onLearnNext(): void {
        if (learnIndex + 1 < DEMO_LEARNING_CONCEPTS.length) {
            learnIndex += 1;
        } else {
            goTo("topic-learned");
        }
    }

    // The demo never touches the backend, so a graded card's answer is faked: it
    // echoes the rating as a plausible next interval for the grading bar to show.
    // Scene navigation is driven by each card's onDone below, so the result's
    // stage fields are cosmetic here.
    const DEMO_INTERVALS: Record<Rating, string> = {
        1: "10m",
        2: "2d",
        3: "4d",
        4: "9d",
    };
    function demoAnswer(
        state: AnswerResult["state"],
        rating: Rating,
    ): Promise<AnswerResult> {
        return Promise.resolve({
            state,
            upgraded: false,
            from: state,
            to: state,
            intervalText: DEMO_INTERVALS[rating],
        });
    }

    // Arrow keys step between scenes, but never while typing in a card's field.
    function onKeydown(event: KeyboardEvent): void {
        if (event.metaKey || event.ctrlKey || event.altKey) {
            return;
        }
        const target = event.target as HTMLElement | null;
        if (target && (target.tagName === "TEXTAREA" || target.tagName === "INPUT")) {
            return;
        }
        if (event.key === "ArrowLeft") {
            event.preventDefault();
            goPrev();
        } else if (event.key === "ArrowRight") {
            event.preventDefault();
            goNext();
        }
    }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="demo">
    <div class="shell">
        <aside class="rail">
            <div class="rail-head">
                <p class="eyebrow">Demo · mock data</p>
                <h1 class="rail-title">Study screens</h1>
                <p class="rail-note">
                    No collection and no cards. Every screen is driven by fixed data.
                </p>
            </div>
            <nav class="scene-list" aria-label="Demo scenes">
                {#each scenes as s, i (s.id)}
                    <button
                        class="scene"
                        class:active={i === sceneIndex}
                        aria-current={i === sceneIndex ? "true" : undefined}
                        on:click={() => enter(i)}
                    >
                        <span class="scene-index">
                            {String(i + 1).padStart(2, "0")}
                        </span>
                        <span class="scene-label">{s.label}</span>
                    </button>
                {/each}
            </nav>
        </aside>

        <main class="main">
            <div class="bar">
                <div class="stepper">
                    <button
                        class="ctl"
                        type="button"
                        on:click={goPrev}
                        disabled={sceneIndex === 0}
                    >
                        <span aria-hidden="true">←</span>
                        Prev
                    </button>
                    <span class="pos">{sceneIndex + 1} / {scenes.length}</span>
                    <button
                        class="ctl"
                        type="button"
                        on:click={goNext}
                        disabled={sceneIndex === scenes.length - 1}
                    >
                        Next <span aria-hidden="true">→</span>
                    </button>
                </div>
                {#if sceneAction}
                    <button
                        class="ctl action"
                        type="button"
                        on:click={() => enter(sceneIndex)}
                    >
                        {sceneAction}
                    </button>
                {/if}
            </div>

            <p class="caption">{scene.caption}</p>

            <div class="stage" class:stage--center={!isFull}>
                {#key stageKey}
                    {#if scene.id === "decks"}
                        <DecksView
                            decks={DEMO_DECKS}
                            onStudy={() => goTo("study")}
                            onDetails={() => goTo("builder")}
                            onCreate={() => goTo("builder")}
                            onDelete={() => Promise.resolve()}
                        />
                    {:else if scene.id === "decks-empty"}
                        <DecksView
                            decks={[]}
                            onStudy={() => {}}
                            onDetails={() => {}}
                            onCreate={() => goTo("builder")}
                            onDelete={() => Promise.resolve()}
                        />
                    {:else if scene.id === "builder"}
                        <HierarchyEditor
                            hierarchy={authoring}
                            persist={false}
                            onBack={() => goTo("decks")}
                        />
                    {:else if scene.id === "concept"}
                        <div class="card-frame">
                            {#if conceptDraft}
                                <ConceptModal
                                    concept={conceptDraft}
                                    onChange={() => {}}
                                />
                            {/if}
                        </div>
                    {:else if scene.id === "study"}
                        <StudyOverview
                            {...DEMO_STUDY}
                            onStart={() => goTo("intro")}
                            onBack={() => goTo("decks")}
                            onAction={() => {}}
                        />
                    {:else if scene.id === "study-empty"}
                        <StudyOverview
                            {...DEMO_STUDY_NOTSTARTED}
                            onStart={() => {}}
                            onBack={() => goTo("decks")}
                            onAction={() => {}}
                        />
                    {:else if scene.id === "intro"}
                        <NewTopicIntro
                            root={DEMO_HIERARCHY.root}
                            topicNodeId={DEMO_TOPIC_NODE_ID}
                            onStart={() => goTo("learn")}
                        />
                    {:else if scene.id === "learn"}
                        <ConceptLearn
                            root={DEMO_HIERARCHY.root}
                            concept={learnConcept}
                            current={learnIndex + 1}
                            total={DEMO_LEARNING_CONCEPTS.length}
                            onNext={onLearnNext}
                        />
                    {:else if scene.id === "topic-learned"}
                        <TopicLearned
                            topic="Cardiovascular"
                            items={DEMO_LEARNING_CONCEPTS.map((c) => ({
                                title: c.title,
                            }))}
                            from="learning"
                            to="practicing"
                            onDone={() => goTo("practice")}
                        />
                    {:else if scene.id === "practice"}
                        <PracticeRecall
                            concept={featured}
                            answer={(rating) => demoAnswer("practicing", rating)}
                            onDone={() => goTo("levelup-pg")}
                        />
                    {:else if scene.id === "levelup-pg"}
                        <UpgradeAnimation
                            concept={featured.title}
                            from="practicing"
                            to="hierarchy"
                            onDone={() => goTo("guided")}
                        />
                    {:else if scene.id === "guided"}
                        <GuidedCard
                            root={DEMO_HIERARCHY.root}
                            concept={featured}
                            conceptId={DEMO_CONCEPT_ID}
                            problem={featuredProblem}
                            answer={(rating) => demoAnswer("hierarchy", rating)}
                            onDone={() => goTo("levelup-gs")}
                        />
                    {:else if scene.id === "levelup-gs"}
                        <UpgradeAnimation
                            concept={featured.title}
                            from="hierarchy"
                            to="mastering"
                            onDone={() => goTo("solo")}
                        />
                    {:else if scene.id === "solo"}
                        <ProblemCard
                            concept={featured}
                            problem={featuredProblem}
                            state="mastering"
                            answer={(rating) => demoAnswer("mastering", rating)}
                            onDone={() => goTo("done")}
                        />
                    {:else if scene.id === "done"}
                        <div class="message">
                            <p class="done-eyebrow">Session complete</p>
                            <p class="message-title">Nothing due right now</p>
                            <p class="muted">
                                You have cleared every card the scheduler had ready.
                            </p>
                            <div class="actions">
                                <button
                                    class="btn primary"
                                    type="button"
                                    on:click={() => goTo("study")}
                                >
                                    Back to overview
                                </button>
                                <button
                                    class="btn"
                                    type="button"
                                    on:click={() => goTo("decks")}
                                >
                                    All decks
                                </button>
                            </div>
                        </div>
                    {/if}
                {/key}
            </div>
        </main>
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;
    @use "$lib/sass/speedrun-synapse" as syn;

    // One token host for the whole demo. The reused screens redeclare the same
    // --sr-* on their own roots; the session cards and the concept editor inherit
    // them from here.
    .demo {
        box-sizing: border-box;
        min-height: 100vh;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;

        @include sr.tokens;
    }

    .shell {
        min-height: 100vh;
        display: grid;
        grid-template-columns: 15rem 1fr;
    }

    // Left rail: the numbered scene index.
    .rail {
        box-sizing: border-box;
        // Without this the phone rail's horizontal scene list (a nowrap row) sets
        // the grid item's min-content, blowing the single 1fr column past the
        // viewport; min-width:0 lets the column shrink and the list scroll.
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        padding: 1.75rem 1.25rem;
        border-right: 1px solid var(--sr-line);
        background: var(--sr-panel-2);
    }
    .rail-head {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }
    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .rail-title {
        margin: 0;
        font-size: 1.35rem;
        font-weight: 720;
        letter-spacing: -0.02em;
    }
    .rail-note {
        margin: 0;
        font-size: 0.82rem;
        color: var(--sr-ink-2);
        max-width: 22ch;
    }

    .scene-list {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }
    .scene {
        appearance: none;
        display: flex;
        align-items: center;
        gap: 0.6rem;
        width: 100%;
        text-align: left;
        padding: 0.5rem 0.6rem;
        border: none;
        border-left: 2px solid transparent;
        border-radius: 0 6px 6px 0;
        background: transparent;
        color: var(--sr-ink-2);
        font: inherit;
        cursor: pointer;
        transition:
            color 0.12s ease,
            background 0.12s ease;
    }
    .scene:hover {
        background: var(--sr-panel);
        color: var(--sr-ink);
    }
    .scene.active {
        background: var(--sr-panel);
        color: var(--sr-ink);
        border-left-color: var(--sr-signal);
    }
    .scene-index {
        font-family: var(--sr-mono);
        font-size: 0.72rem;
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink-3);
    }
    .scene.active .scene-index {
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .scene.active .scene-index {
        color: var(--sr-signal);
    }
    .scene-label {
        font-size: 0.9rem;
        font-weight: 520;
    }

    // Right pane: the toolbar, the caption, and the live stage.
    .main {
        display: flex;
        flex-direction: column;
        min-width: 0;
        min-height: 100vh;
    }
    .bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        flex-wrap: wrap;
        padding: 1rem 1.5rem;
        border-bottom: 1px solid var(--sr-line);
    }
    .stepper {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    .ctl {
        appearance: none;
        display: inline-flex;
        align-items: center;
        gap: 0.3rem;
        padding: 0.4rem 0.75rem;
        border: 1px solid var(--sr-line-2);
        border-radius: 7px;
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        cursor: pointer;
        transition:
            color 0.12s ease,
            border-color 0.12s ease,
            background 0.12s ease;
    }
    .ctl:hover:not(:disabled) {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    .ctl:disabled {
        opacity: 0.45;
        cursor: default;
    }
    .ctl.action {
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
        background: var(--sr-signal-weak);
    }
    :global(.night-mode) .ctl.action {
        color: var(--sr-signal);
    }
    .pos {
        font-family: var(--sr-mono);
        font-size: 11px;
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink-3);
        padding: 0 0.25rem;
    }

    .caption {
        margin: 0;
        padding: 0.75rem 1.5rem 0;
        font-family: var(--sr-mono);
        font-size: 0.78rem;
        letter-spacing: 0.02em;
        color: var(--sr-ink-3);
    }

    .stage {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
    }
    // Session cards and the concept editor centre; full-bleed screens fill it.
    .stage--center {
        align-items: center;
        justify-content: center;
        gap: 1rem;
        padding: 2rem 1.5rem 3rem;
    }

    // Standalone concept editor: framed as its builder column would be.
    .card-frame {
        box-sizing: border-box;
        width: 100%;
        max-width: 40rem;
        padding: 24px 26px;
        @include syn.card;
    }

    // Finished state, mirroring the real orchestrator's copy.
    .message {
        max-width: 34rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        text-align: center;
    }
    .done-eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .done-eyebrow {
        color: var(--sr-signal);
    }
    .muted {
        margin: 0;
        color: var(--sr-ink-3);
        font-family: var(--sr-mono);
        font-size: 0.8rem;
        letter-spacing: 0.04em;
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
    .btn:hover {
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    .btn.primary {
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
    }
    .btn.primary:hover {
        filter: brightness(1.04);
    }

    // Keyboard focus is always visible, on the chrome and the reused components.
    .demo :global(:focus-visible) {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 6px;
    }

    @media (prefers-reduced-motion: reduce) {
        .scene,
        .ctl {
            transition: none;
        }
    }

    @media (max-width: 48rem) {
        .shell {
            grid-template-columns: 1fr;
        }
        .rail {
            border-right: none;
            border-bottom: 1px solid var(--sr-line);
        }
        .scene-list {
            flex-direction: row;
            flex-wrap: nowrap;
            overflow-x: auto;
            gap: 0.4rem;
            padding-bottom: 0.25rem;
        }
        .scene {
            flex: 0 0 auto;
            border-left: none;
            border-bottom: 2px solid transparent;
            border-radius: 6px;
        }
        .scene.active {
            border-left-color: transparent;
            border-bottom-color: var(--sr-signal);
        }
        .scene-label {
            white-space: nowrap;
        }
    }
</style>
