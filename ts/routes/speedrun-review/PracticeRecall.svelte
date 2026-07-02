<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import DifficultyBar from "./DifficultyBar.svelte";
    import { type Concept, type Rating } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import ReviewCard from "./ReviewCard.svelte";

    export let concept: Concept;
    export let onRate: (rating: Rating) => void;

    let revealed = false;
    let answer = "";

    // Fresh card, fresh recall attempt.
    $: if (concept) {
        revealed = false;
        answer = "";
    }

    $: cue = concept.title.trim() || "Untitled concept";
    $: content = concept.content.trim();
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge state="practicing" />
    </svelte:fragment>

    <p class="eyebrow">Recall</p>
    <h1 class="cue">{cue}</h1>

    <label class="field">
        <span class="label">Say it in your own words</span>
        <textarea
            class="text"
            rows="3"
            bind:value={answer}
            placeholder="Explain the concept from memory"
        ></textarea>
    </label>

    {#if revealed}
        <div class="reveal">
            <p class="label">The concept</p>
            {#if content}
                <p class="body">{content}</p>
            {:else}
                <p class="body none">No description was authored for this concept.</p>
            {/if}
        </div>
    {/if}

    <svelte:fragment slot="footer">
        {#if revealed}
            <DifficultyBar {onRate} />
        {:else}
            <button class="btn" on:click={() => (revealed = true)}>Show answer</button>
        {/if}
    </svelte:fragment>
</ReviewCard>

<style lang="scss">
    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .cue {
        margin: 0;
        font-size: 1.6rem;
        font-weight: 680;
        letter-spacing: -0.015em;
        line-height: 1.15;
    }
    .field {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }
    .label {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.13em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .text {
        width: 100%;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel-2);
        color: var(--sr-ink);
        border-radius: 6px;
        padding: 0.55rem 0.65rem;
        font: inherit;
        resize: vertical;
        min-height: 4rem;
    }
    .text:focus {
        outline: none;
        border-color: var(--sr-signal-line);
        background: var(--sr-panel);
    }
    .text::placeholder {
        color: var(--sr-ink-3);
    }
    .reveal {
        border-top: 1px solid var(--sr-line);
        padding-top: 0.9rem;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    .body {
        margin: 0;
        line-height: 1.6;
        max-width: 42rem;
    }
    .body.none {
        color: var(--sr-ink-3);
        font-style: italic;
    }
    .btn {
        appearance: none;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        padding: 0.6rem 1.4rem;
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
    @media (max-width: 34rem) {
        .cue {
            font-size: 1.35rem;
        }
    }
</style>
