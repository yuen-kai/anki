<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";

    import { findConcept, type Node, scaffoldSteps } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import ReviewCard from "./ReviewCard.svelte";

    export let root: Node;
    export let conceptId: string;
    export let onComplete: () => void;

    let stepIndex = 0;
    let wrongId: string | null = null;
    let chosen: string[] = [];

    // Rebuild when the orchestrator hands over a new concept.
    $: steps = scaffoldSteps(root, conceptId);
    $: concept = findConcept(root, conceptId);
    $: if (conceptId) {
        stepIndex = 0;
        wrongId = null;
        chosen = [];
    }
    $: step = steps[stepIndex] ?? null;

    // Nothing to place (concept sits on the root): let the caller move on.
    onMount(() => {
        if (steps.length === 0) {
            onComplete();
        }
    });

    function choose(id: string, title: string): void {
        if (!step) {
            return;
        }
        if (id === step.correctId) {
            wrongId = null;
            chosen = [...chosen, title];
            if (stepIndex + 1 >= steps.length) {
                onComplete();
            } else {
                stepIndex += 1;
            }
        } else {
            // No authored cue: a wrong branch simply re-prompts this level.
            wrongId = id;
        }
    }
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge state="hierarchy" />
    </svelte:fragment>

    <p class="eyebrow">Locate</p>
    <h1 class="title">{concept?.title.trim() || "Untitled concept"}</h1>
    <p class="hint">Pick the branch it belongs to, level by level.</p>

    {#if chosen.length}
        <nav class="trail" aria-label="Chosen path">
            {#each chosen as crumb, i (i)}
                {#if i > 0}<span class="sep" aria-hidden="true">›</span>{/if}
                <span class="crumb">{crumb}</span>
            {/each}
        </nav>
    {/if}

    {#if step}
        <ul class="options">
            {#each step.options as option (option.id)}
                <li>
                    <button
                        class="option"
                        class:wrong={wrongId === option.id}
                        on:click={() => choose(option.id, option.title)}
                    >
                        {option.title.trim() || "Untitled topic"}
                    </button>
                </li>
            {/each}
        </ul>
        {#if wrongId}
            <p class="miss" role="status">Not there. Try another branch.</p>
        {/if}
    {/if}

    <svelte:fragment slot="footer">
        {#if steps.length}
            <span class="progress">Step {stepIndex + 1} / {steps.length}</span>
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
    .title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 680;
        letter-spacing: -0.015em;
    }
    .hint {
        margin: 0;
        color: var(--sr-ink-2);
        font-size: 0.95rem;
    }
    .trail {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.85rem;
    }
    .crumb {
        font-family: var(--sr-mono);
        color: var(--sr-ink-2);
        background: var(--sr-panel-2);
        border: 1px solid var(--sr-line);
        border-radius: 5px;
        padding: 0.1rem 0.45rem;
    }
    .sep {
        color: var(--sr-ink-3);
    }
    .options {
        list-style: none;
        margin: 0;
        padding: 0;
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
        gap: 0.5rem;
    }
    .option {
        width: 100%;
        padding: 0.7rem 0.8rem;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        background: var(--sr-panel);
        color: var(--sr-ink);
        font: inherit;
        font-weight: 560;
        text-align: left;
        cursor: pointer;
        transition:
            border-color 0.12s ease,
            background 0.12s ease,
            transform 0.08s ease;
    }
    .option:hover {
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    .option.wrong {
        border-color: var(--sr-line-2);
        opacity: 0.6;
        animation: sr-shake 0.24s ease;
    }
    @keyframes sr-shake {
        0%,
        100% {
            transform: translateX(0);
        }
        25% {
            transform: translateX(-3px);
        }
        75% {
            transform: translateX(3px);
        }
    }
    @media (prefers-reduced-motion: reduce) {
        .option.wrong {
            animation: none;
        }
    }
    .miss {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 0.78rem;
        color: var(--sr-ink-2);
    }
    .progress {
        font-family: var(--sr-mono);
        font-size: 0.78rem;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }
</style>
