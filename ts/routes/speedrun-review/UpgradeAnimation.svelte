<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onDestroy, onMount } from "svelte";

    import { upgradeLabels } from "./lib";

    export let from: string;
    export let to: string;
    export let onDone: () => void;

    $: labels = upgradeLabels(from, to);

    const reduced =
        typeof window !== "undefined" &&
        window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    // Hold the moment long enough to register, but never block the session.
    const HOLD_MS = reduced ? 1100 : 2200;

    let fired = false;
    let timer: ReturnType<typeof setTimeout>;

    function finish(): void {
        if (fired) {
            return;
        }
        fired = true;
        clearTimeout(timer);
        onDone();
    }

    function onKey(event: KeyboardEvent): void {
        if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            finish();
        }
    }

    onMount(() => {
        timer = setTimeout(finish, HOLD_MS);
    });
    onDestroy(() => clearTimeout(timer));
</script>

<svelte:window on:keydown={onKey} />

<div class="upgrade" class:reduced>
    <p class="eyebrow">Mastery up</p>
    <div class="jump">
        <span class="stage from">{labels.from}</span>
        <span class="arrow" aria-hidden="true">→</span>
        <span class="stage to">
            {labels.to}
            <span class="burst" aria-hidden="true"></span>
        </span>
    </div>
    <button class="btn" on:click={finish}>Continue</button>
</div>

<style lang="scss">
    .upgrade {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.75rem;
        text-align: center;
    }
    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 12px;
        font-weight: 600;
        letter-spacing: 0.22em;
        text-transform: uppercase;
        color: var(--sr-signal-ink);
        animation: sr-fade 320ms ease both;
    }
    :global(.night-mode) .eyebrow {
        color: var(--sr-signal);
    }

    .jump {
        display: flex;
        align-items: center;
        gap: 1.1rem;
        font-size: 1.9rem;
        font-weight: 700;
        letter-spacing: -0.01em;
    }
    .stage.from {
        color: var(--sr-ink-3);
    }
    .arrow {
        color: var(--sr-ink-3);
        font-size: 1.5rem;
    }
    .stage.to {
        position: relative;
        color: var(--sr-signal-ink);
        animation: sr-pop 480ms cubic-bezier(0.2, 0.9, 0.3, 1.2) both;
        animation-delay: 160ms;
    }
    :global(.night-mode) .stage.to {
        color: var(--sr-signal);
    }
    // A single marigold ring that expands and fades behind the new stage.
    .burst {
        position: absolute;
        inset: -0.6rem -0.9rem;
        border: 2px solid var(--sr-signal);
        border-radius: 999px;
        opacity: 0;
        animation: sr-burst 720ms ease-out both;
        animation-delay: 220ms;
    }

    @keyframes sr-fade {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }
    @keyframes sr-pop {
        0% {
            opacity: 0;
            transform: scale(0.7);
        }
        60% {
            transform: scale(1.08);
        }
        100% {
            opacity: 1;
            transform: scale(1);
        }
    }
    @keyframes sr-burst {
        0% {
            opacity: 0.7;
            transform: scale(0.6);
        }
        100% {
            opacity: 0;
            transform: scale(1.4);
        }
    }

    // Reduced motion: keep the message, drop the movement.
    .reduced .eyebrow,
    .reduced .stage.to {
        animation: none;
    }
    .reduced .burst {
        display: none;
    }

    .btn {
        appearance: none;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        padding: 0.6rem 1.5rem;
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
        .jump {
            font-size: 1.4rem;
            gap: 0.7rem;
        }
    }
</style>
