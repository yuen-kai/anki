<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { type Rating, RATINGS } from "./lib";

    // Grades the card through FSRS. Placed inline at the bottom-center of the
    // card, standing in for where a Next button would sit.
    export let onRate: (rating: Rating) => void;
    // While inactive the keyboard shortcuts are ignored (e.g. before reveal).
    export let active = true;

    function rate(rating: Rating): void {
        if (active) {
            onRate(rating);
        }
    }

    // 1..4 grade the card, matching base Anki's ease shortcuts. Typing in a field
    // takes precedence so a free-response answer is never eaten.
    function onKey(event: KeyboardEvent): void {
        if (!active || event.metaKey || event.ctrlKey || event.altKey) {
            return;
        }
        const target = event.target as HTMLElement | null;
        if (target && (target.tagName === "TEXTAREA" || target.tagName === "INPUT")) {
            return;
        }
        const n = Number(event.key);
        if (n >= 1 && n <= 4) {
            event.preventDefault();
            rate(n as Rating);
        }
    }
</script>

<svelte:window on:keydown={onKey} />

<div class="bar" role="group" aria-label="Rate difficulty">
    {#each RATINGS as choice (choice.rating)}
        <button
            class="ease"
            class:good={choice.rating === 3}
            on:click={() => rate(choice.rating)}
            disabled={!active}
        >
            <span class="name">{choice.label}</span>
            <span class="key" aria-hidden="true">{choice.rating}</span>
        </button>
    {/each}
</div>

<style lang="scss">
    .bar {
        display: flex;
        justify-content: center;
        flex-wrap: wrap;
        gap: 0.5rem;
    }
    .ease {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        min-width: 5.5rem;
        justify-content: center;
        padding: 0.6rem 0.9rem;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        background: var(--sr-panel);
        color: var(--sr-ink);
        font: inherit;
        font-weight: 560;
        cursor: pointer;
        transition:
            border-color 0.12s ease,
            background 0.12s ease;
    }
    .ease:hover:not(:disabled) {
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    // Good is the expected pass, so it carries the single marigold hint; the
    // others stay quiet ink so the row never turns into a colour swatch.
    .ease.good {
        border-color: var(--sr-signal-line);
    }
    .ease.good:hover:not(:disabled) {
        background: var(--sr-signal-weak);
    }
    .ease:disabled {
        opacity: 0.45;
        cursor: default;
    }
    .name {
        font-size: 0.95rem;
    }
    .key {
        font-family: var(--sr-mono);
        font-size: 0.7rem;
        color: var(--sr-ink-3);
        border: 1px solid var(--sr-line-2);
        border-radius: 4px;
        padding: 0 0.28rem;
        line-height: 1.3;
    }
    @media (max-width: 34rem) {
        .ease {
            flex: 1 1 40%;
            min-width: 0;
        }
        .key {
            display: none;
        }
    }
</style>
