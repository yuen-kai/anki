<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Learn (first acquisition): two worked problems shown with only their correct
answers, the learner names what they share, then the concept detail reveals
below on the same card (via the seam). No difficulty rating — the action is
"Next concept".
-->
<script lang="ts">
    import { type Concept, type Node, pathToConcept, pickTwoProblems } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import MediaImage from "./MediaImage.svelte";
    import ReviewCard from "./ReviewCard.svelte";
    import Seam from "./Seam.svelte";

    export let concept: Concept;
    // The authored tree, for the topic name in the header and the group/topic
    // breadcrumb in the detail. Optional so the card still renders without it.
    export let root: Node | null = null;
    // The ordinal of this concept within its topic block, e.g. 3 of 5.
    export let current: number;
    export let total: number;
    export let onNext: () => void;

    const CASE_LABELS = ["A", "B"];

    let revealed = false;
    let shared = "";

    // Reset when the orchestrator swaps in the next concept of the block.
    $: if (concept) {
        revealed = false;
        shared = "";
    }

    $: shown = pickTwoProblems(concept);
    $: title = concept.title.trim() || "Untitled concept";
    $: content = concept.content.trim();
    // [root, ...groups, topic] down to the leaf holding this concept.
    $: path = root ? (pathToConcept(root, concept.id) ?? []) : [];
    $: topicName = path.length ? path[path.length - 1].title.trim() || "Topic" : "";
    // The group · topic trail (everything under the deck root).
    $: trail = path
        .slice(1)
        .map((node) => node.title.trim() || "Untitled")
        .join(" · ");
    // Concepts still to learn in this topic block, this one included.
    $: left = Math.max(1, total - current + 1);
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge state="learning" />
        {#if topicName}
            <span class="count-phone">{topicName} · {left}/{total} left</span>
        {/if}
    </svelte:fragment>

    <div class="sc-sec">
        <div class="step-row">
            <p class="step">Step ①</p>
            {#if topicName}
                <span class="count-pill">
                    {topicName} · {left} of {total} concepts left
                </span>
            {/if}
        </div>
        <p class="ask">What do these two problems share?</p>

        {#each shown as problem, i (problem.id)}
            <div class="case">
                <p class="case-label">Problem {CASE_LABELS[i] ?? i + 1}</p>
                <p class="case-prompt">{problem.prompt || "Untitled problem"}</p>
                <MediaImage
                    filename={problem.image}
                    alt="Problem {CASE_LABELS[i] ?? i + 1} figure"
                />
                {#if problem.answer}
                    <p class="case-answer">
                        <span aria-hidden="true">✓</span>
                        {problem.answer}
                    </p>
                {:else}
                    <p class="case-answer none">Answer not marked</p>
                {/if}
            </div>
        {/each}

        <div class="fill">
            <span class="fill-lead">Both point to</span>
            <input
                class="fill-input"
                bind:value={shared}
                placeholder="the shared idea"
                aria-label="The idea both problems share"
            />
        </div>

        <button class="reveal-btn" type="button" on:click={() => (revealed = true)}>
            Reveal concept details
        </button>
    </div>

    {#if revealed}
        <Seam label="details appear below · same screen" />
        <div class="sc-sec sc-sec--reveal">
            <p class="step green">Step ② · Concept details</p>
            <h2 class="concept-title">{title}</h2>
            {#if trail}
                <p class="breadcrumb">{trail}</p>
            {/if}
            {#if content}
                <p class="concept-body">{content}</p>
            {:else}
                <p class="concept-body none">
                    No description was authored for this concept.
                </p>
            {/if}
            <MediaImage filename={concept.image} alt={title} />
            <button class="next-btn" type="button" on:click={onNext}>
                Next concept
            </button>
        </div>
    {/if}
</ReviewCard>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;
    @use "./sr-tokens" as srt;

    // The topic-block count: a practice-tinted pill on the Step ① row (desktop
    // 2c), or a mono meta in the header on phone (2g). One is shown per width.
    .count-phone {
        display: none;
        margin-left: auto;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 9px;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
        white-space: nowrap;
    }
    .step-row {
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .count-pill {
        margin-left: auto;
        padding: 3px 9px;
        border-radius: var(--sr-radius-pill);
        background: var(--sr-stage-practice-soft);
        color: var(--sr-stage-practice-deep);
        font-family: var(--sr-mono);
        font-weight: 600;
        font-size: 8.5px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        white-space: nowrap;
        font-variant-numeric: tabular-nums;
    }

    .step {
        @include srt.step;
    }
    .step.green {
        --step-color: var(--sr-stage-solo-deep);
    }
    .ask {
        margin: 10px 0;
        font-size: 12.5px;
        font-weight: 600;
        color: var(--sr-ink);
    }

    .case {
        background: var(--sr-white);
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-tile);
        padding: 10px 12px;
        margin-bottom: 7px;
    }
    .case-label {
        margin: 0;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 8.5px;
        letter-spacing: var(--sr-track-tag);
        text-transform: uppercase;
        color: var(--sr-faint);
    }
    .case-prompt {
        margin: 4px 0 0;
        font-size: 11.5px;
        line-height: 1.45;
        color: var(--sr-ink-2);
    }
    .case-answer {
        display: inline-flex;
        align-items: center;
        gap: 5px;
        margin: 7px 0 0;
        padding: 6px 9px;
        border-radius: var(--sr-radius-chip);
        background: var(--sr-stage-solo-soft);
        font-size: 11px;
        font-weight: 600;
        color: var(--sr-stage-solo-deep);
    }
    .case-answer.none {
        background: var(--sr-inset);
        color: var(--sr-ink-3);
        font-style: italic;
        font-weight: 500;
    }

    // The learner names the shared idea: "Both point to ___", coral caret.
    .fill {
        display: flex;
        align-items: baseline;
        gap: 6px;
        flex-wrap: wrap;
        margin-top: 10px;
        padding: 10px 12px;
        background: var(--sr-white);
        border: 1.5px solid var(--sr-line-strong);
        border-radius: var(--sr-radius-tile);
        font-size: 12px;
        color: var(--sr-ink);
    }
    .fill-lead {
        flex-shrink: 0;
    }
    .fill-input {
        flex: 1;
        min-width: 8rem;
        border: none;
        border-bottom: 2px solid var(--sr-signal);
        background: transparent;
        color: var(--sr-ink);
        caret-color: var(--sr-signal);
        font: inherit;
        padding: 0 0 2px;
    }
    .fill-input:focus {
        outline: none;
    }
    .fill-input::placeholder {
        color: var(--sr-faint);
    }

    .reveal-btn {
        @include syn.btn;
        @include syn.btn-primary;
        @include syn.btn-block;
        margin-top: 12px;
    }

    .concept-title {
        margin: 0;
        font-size: 15px;
        font-weight: 700;
        color: var(--sr-ink);
    }
    .breadcrumb {
        margin: 3px 0 0;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 9px;
        letter-spacing: var(--sr-track-tag);
        text-transform: uppercase;
        color: var(--sr-faint);
    }
    .concept-body {
        margin: 10px 0 0;
        font-size: 12px;
        line-height: 1.55;
        color: var(--sr-ink);
    }
    .concept-body.none {
        color: var(--sr-ink-3);
        font-style: italic;
    }
    .next-btn {
        @include syn.btn;
        @include syn.btn-dark;
        @include syn.btn-block;
        margin-top: 12px;
    }

    // Phone (2g): the count moves up to the header; the desktop pill hides.
    @media (max-width: 34rem) {
        .count-phone {
            display: block;
        }
        .count-pill {
            display: none;
        }
    }
</style>
