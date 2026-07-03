<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Practice (free recall): given the concept title, the learner recites the
description from memory, then the saved description drops in below on the same
card (via the seam) to self-check against. Grading rides at the bottom.
-->
<script lang="ts">
    import DifficultyBar from "./DifficultyBar.svelte";
    import { type AnswerResult, type Concept, type Rating } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import ReviewCard from "./ReviewCard.svelte";
    import Seam from "./Seam.svelte";

    export let concept: Concept;
    export let answer: (rating: Rating) => Promise<AnswerResult>;
    export let onDone: (result: AnswerResult) => void;
    export let onError: (err: unknown) => void = () => {};

    let revealed = false;
    let recall = "";

    // Fresh card, fresh recall attempt.
    $: if (concept) {
        revealed = false;
        recall = "";
    }

    $: title = concept.title.trim() || "Untitled concept";
    $: content = concept.content.trim();
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge state="practicing" />
    </svelte:fragment>

    <div class="sc-sec">
        <p class="step">Step ① · Recall from the title</p>
        <h2 class="cue">{title}</h2>
        <p class="field-label">Your recall</p>
        <textarea
            class="recall"
            rows="3"
            bind:value={recall}
            placeholder="Say the description from memory"
        ></textarea>
        <button class="check-btn" type="button" on:click={() => (revealed = true)}>
            Check against description
        </button>
    </div>

    {#if revealed}
        <Seam label="saved text drops in below · same screen" />
        <div class="sc-sec sc-sec--reveal">
            <p class="step green">Step ② · Self-check</p>
            <p class="field-label">Saved description</p>
            {#if content}
                <p class="saved">{content}</p>
            {:else}
                <p class="saved none">No description was authored for this concept.</p>
            {/if}
        </div>

        <Seam label="same screen · now rate the card" />
        <div class="sc-sec sc-sec--grade">
            <p class="step">Step ③ · Rate difficulty</p>
            <DifficultyBar {answer} {onDone} {onError} />
        </div>
    {/if}
</ReviewCard>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;
    @use "./sr-tokens" as srt;

    .step {
        @include srt.step;
    }
    .step.green {
        --step-color: var(--sr-stage-solo-deep);
    }
    .cue {
        margin: 9px 0 0;
        font-size: 18px;
        font-weight: 700;
        letter-spacing: var(--sr-tighten);
        color: var(--sr-ink);
    }
    .field-label {
        margin: 12px 0 6px;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 9px;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--sr-faint);
    }
    .recall {
        width: 100%;
        box-sizing: border-box;
        background: var(--sr-white);
        border: 1.5px solid var(--sr-line-strong);
        border-radius: var(--sr-radius-tile);
        padding: 11px 13px;
        font-family: var(--sr-sans);
        font-size: 12px;
        line-height: 1.5;
        color: var(--sr-ink);
        caret-color: var(--sr-signal);
        resize: vertical;
        min-height: 3.4rem;
    }
    .recall:focus {
        outline: none;
        border-color: var(--sr-signal);
    }
    .recall::placeholder {
        color: var(--sr-faint);
    }
    .check-btn {
        @include syn.btn;
        @include syn.btn-primary;
        @include syn.btn-block;
        margin-top: 12px;
    }
    .saved {
        @include syn.explain;
        margin: 0;
    }
    .saved.none {
        color: var(--sr-ink-3);
        font-style: italic;
    }
</style>
