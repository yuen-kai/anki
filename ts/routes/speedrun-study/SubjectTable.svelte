<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { SubjectBreakdown } from "./lib";

    export let subjects: SubjectBreakdown;
    export let error: string | null = null;
    // The subject holding readiness back the most, or null when there is not
    // enough evidence to name one.
    export let bottleneckId: string | null = null;

    const pct = (fraction: number): string => `${Math.round(fraction * 100)}%`;
</script>

<h2 class="eyebrow">Per subject</h2>

{#if error}
    <p class="state">Couldn't load the subject breakdown. {error}</p>
{:else}
    <div class="cells">
        {#each subjects.subjects as s (s.id)}
            {@const flag = s.id === bottleneckId}
            <div class="cell">
                <div class="cell-name">
                    {s.name}
                    {#if flag}<span class="sr-only">
                            (most holding readiness back)
                        </span>{/if}
                </div>
                <div class="rows">
                    <span class="lbl">Memory</span>
                    <span class="bar">
                        <span
                            class="fill mem"
                            style="width:{s.hasMemoryData
                                ? Math.round(s.meanRetrievability * 100)
                                : 0}%"
                        ></span>
                    </span>
                    <span class="val">
                        {s.hasMemoryData ? pct(s.meanRetrievability) : "\u2014"}
                    </span>

                    <span class="lbl">Perform</span>
                    <span class="bar">
                        <span
                            class="fill perf"
                            style="width:{s.hasApplicationData
                                ? Math.round(s.applicationAccuracy * 100)
                                : 0}%"
                        ></span>
                    </span>
                    <span class="val">
                        {s.hasApplicationData ? pct(s.applicationAccuracy) : "\u2014"}
                    </span>

                    <span class="lbl">Coverage</span>
                    <span class="bar">
                        <span
                            class="fill cov"
                            class:flag
                            style="width:{Math.round(s.coverage * 100)}%"
                        ></span>
                    </span>
                    <span class="val" class:flag>{pct(s.coverage)}</span>
                </div>
            </div>
        {/each}
    </div>
{/if}

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .eyebrow {
        margin: 0 0 14px;
        @include syn.eyebrow;
    }

    .state {
        margin: 0;
        font-size: 0.9rem;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }

    .cells {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 14px;
    }
    .cell {
        box-sizing: border-box;
        padding: 13px 15px;
        @include syn.inset;
    }
    .cell-name {
        margin-bottom: 9px;
        font-weight: 700;
        font-size: 12.5px;
    }

    .rows {
        display: grid;
        grid-template-columns: 74px 1fr 38px;
        gap: 6px 10px;
        align-items: center;
    }
    .lbl {
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 9.5px;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .bar {
        height: 5px;
        border-radius: 3px;
        background: var(--sr-track);
        overflow: hidden;
    }
    .fill {
        display: block;
        height: 100%;
        border-radius: 3px;
    }
    .fill.mem {
        background: var(--sr-score-memory);
    }
    .fill.perf {
        background: var(--sr-score-performance);
    }
    .fill.cov {
        background: var(--sr-stage-solo);
    }
    // The bottleneck's coverage is the weak point: amber bar, deep-coral figure.
    .fill.cov.flag {
        background: var(--sr-stage-guided);
    }
    .val {
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 9.5px;
        text-align: right;
        color: var(--sr-ink);
        font-variant-numeric: tabular-nums;
    }
    .val.flag {
        color: var(--sr-signal-deep);
    }

    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
    }

    @media (max-width: 40rem) {
        .cells {
            grid-template-columns: 1fr;
        }
    }
</style>
