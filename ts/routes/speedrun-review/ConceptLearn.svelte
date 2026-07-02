<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { type Concept, pickTwoProblems } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import ReviewCard from "./ReviewCard.svelte";

    export let concept: Concept;
    // The ordinal of this concept within its topic block, e.g. 3 of 5.
    export let current: number;
    export let total: number;
    export let onNext: () => void;

    let revealed = false;
    let similarity = "";

    // Reset when the orchestrator swaps in the next concept of the block.
    $: if (concept) {
        revealed = false;
        similarity = "";
    }

    $: shown = pickTwoProblems(concept);
    $: title = concept.title.trim() || "Untitled concept";
    $: content = concept.content.trim();
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge state="learning" />
        <span class="count">{current} / {total} concepts</span>
    </svelte:fragment>

    <p class="eyebrow">New concept</p>
    <h1 class="title">{title}</h1>

    {#if shown.length}
        <div class="cases" class:single={shown.length === 1}>
            {#each shown as problem (problem.id)}
                <div class="case">
                    <p class="prompt">{problem.prompt || "Untitled problem"}</p>
                    {#if problem.answer}
                        <p class="answer">
                            <span class="tick" aria-hidden="true">✓</span>
                            {problem.answer}
                        </p>
                    {:else}
                        <p class="answer none">Answer not marked</p>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}

    <label class="field">
        <span class="label">What these share</span>
        <textarea
            class="text"
            rows="2"
            bind:value={similarity}
            placeholder="Note the common principle behind both"
        ></textarea>
    </label>

    {#if revealed && content}
        <div class="reveal">
            <p class="label">The concept</p>
            <p class="body">{content}</p>
        </div>
    {/if}

    <svelte:fragment slot="footer">
        {#if revealed}
            <button class="btn primary" on:click={onNext}>Continue</button>
        {:else}
            <button class="btn" on:click={() => (revealed = true)}>
                Reveal concept
            </button>
        {/if}
    </svelte:fragment>
</ReviewCard>

<style lang="scss">
    .count {
        font-family: var(--sr-mono);
        font-size: 0.78rem;
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink-3);
    }
    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .eyebrow {
        color: var(--sr-signal);
    }
    .title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 680;
        letter-spacing: -0.015em;
        line-height: 1.15;
    }

    .cases {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 0.75rem;
    }
    .cases.single {
        grid-template-columns: 1fr;
    }
    .case {
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        padding: 0.8rem 0.85rem;
        background: var(--sr-panel-2);
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }
    .prompt {
        margin: 0;
        font-weight: 560;
    }
    .answer {
        margin: 0;
        display: flex;
        align-items: baseline;
        gap: 0.4rem;
        color: var(--sr-signal-ink);
        font-size: 0.95rem;
    }
    :global(.night-mode) .answer {
        color: var(--sr-signal);
    }
    .tick {
        font-size: 0.85rem;
    }
    .answer.none {
        color: var(--sr-ink-3);
        font-style: italic;
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
        padding: 0.5rem 0.6rem;
        font: inherit;
        resize: vertical;
        min-height: 3rem;
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
        color: var(--sr-ink);
        line-height: 1.6;
        max-width: 42rem;
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
    .btn.primary {
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
    }
    .btn.primary:hover {
        filter: brightness(1.04);
        background: var(--sr-signal);
    }

    @media (max-width: 34rem) {
        .cases {
            grid-template-columns: 1fr;
        }
        .title {
            font-size: 1.3rem;
        }
    }
</style>
