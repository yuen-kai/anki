<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The Guided locate step: place the concept in the hierarchy, one level at a time,
as rows of sibling chips (a level's row appears once the level above is placed).
Picking the chip on the path selects it and reveals the next level; the final
placement fires `onComplete` so the Guided card can disclose the question below.
Carries no card chrome — it is a body embedded in the locate section.
-->
<script lang="ts">
    import { onMount } from "svelte";

    import { type Node, scaffoldSteps } from "./lib";

    export let root: Node;
    export let conceptId: string;
    export let onComplete: () => void;

    // The hierarchy has no semantic level names; label rows the way the canvas
    // does (Topic, then Subtopic), falling back for deeper trees.
    const LEVEL_LABELS = ["Topic", "Subtopic", "Section", "Area"];
    function levelLabel(i: number): string {
        return LEVEL_LABELS[i] ?? `Level ${i + 1}`;
    }

    let stepIndex = 0;
    let wrongId: string | null = null;

    // Rebuild when the orchestrator hands over a new concept.
    $: steps = scaffoldSteps(root, conceptId);
    $: if (conceptId) {
        stepIndex = 0;
        wrongId = null;
    }

    // Nothing to place (concept sits on the root): let the caller move straight
    // to the question.
    onMount(() => {
        if (steps.length === 0) {
            onComplete();
        }
    });

    function choose(level: number, id: string): void {
        if (level !== stepIndex) {
            return;
        }
        if (id === steps[level].correctId) {
            wrongId = null;
            if (stepIndex + 1 >= steps.length) {
                stepIndex += 1;
                onComplete();
            } else {
                stepIndex += 1;
            }
        } else {
            wrongId = id;
        }
    }
</script>

<div class="locate">
    {#each steps as step, i (i)}
        {#if i <= stepIndex}
            <div class="row">
                <span class="row-label">{levelLabel(i)}</span>
                {#each step.options as option (option.id)}
                    <button
                        class="chip"
                        class:selected={i < stepIndex && option.id === step.correctId}
                        class:wrong={i === stepIndex && wrongId === option.id}
                        type="button"
                        disabled={i < stepIndex}
                        on:click={() => choose(i, option.id)}
                    >
                        {option.title.trim() || "Untitled"}
                    </button>
                {/each}
            </div>
        {/if}
    {/each}
    {#if wrongId}
        <p class="miss" role="status">Not there. Try another branch.</p>
    {/if}
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .locate {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    .row {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 5px;
    }
    .row-label {
        width: 100%;
        font-family: var(--sr-mono);
        font-weight: 600;
        font-size: 8px;
        letter-spacing: var(--sr-track-tag);
        text-transform: uppercase;
        color: var(--sr-faint);
    }
    .chip {
        @include syn.chip;
        appearance: none;
        cursor: pointer;
        transition:
            border-color 0.12s ease,
            background 0.12s ease,
            transform 0.08s ease;
    }
    .chip:hover:not(:disabled) {
        border-color: var(--sr-line-dashed);
    }
    .chip:disabled {
        cursor: default;
    }
    .chip.selected {
        @include syn.chip-selected;
    }
    .chip.wrong {
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
    .miss {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 9px;
        letter-spacing: 0.04em;
        color: var(--sr-stage-guided-deep);
    }
    @media (prefers-reduced-motion: reduce) {
        .chip {
            transition: none;
        }
        .chip.wrong {
            animation: none;
        }
    }
</style>
