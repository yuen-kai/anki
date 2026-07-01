<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { MASTERY_STAGES, STAGE_COUNT, type TopicsView } from "./lib";

    export let view: TopicsView;
    export let error: string | null = null;
</script>

{#if error}
    <div class="state">
        <p class="headline">Couldn't load topic progress</p>
        <p class="detail">{error}</p>
    </div>
{:else if view.total === 0}
    <div class="state">
        <p class="headline">No topics tracked yet</p>
        <p class="detail">
            Study this deck to start moving its topics up the ladder, from learning
            to mastering.
        </p>
    </div>
{:else}
    <!-- The ladder is the map: every topic climbs these four rungs in order, so
         the summary counts how many sit on each. -->
    <ol class="ladder" aria-label="Topics on each stage of the ladder">
        {#each MASTERY_STAGES as stage, i}
            <li class="rung" class:occupied={view.distribution[i] > 0}>
                <span class="rung-count">{view.distribution[i]}</span>
                <span class="rung-label">{stage.label}</span>
                <span class="rung-blurb">{stage.blurb}</span>
            </li>
        {/each}
    </ol>

    <div class="groups">
        {#each view.groups as group}
            <div class="group">
                <h3 class="group-heading">{group.heading}</h3>
                <ul class="rows">
                    {#each group.topics as topic}
                        <li class="row">
                            <span class="topic-label">{topic.label}</span>
                            <span
                                class="track"
                                role="img"
                                aria-label="{topic.stageLabel}, stage {topic.stage +
                                    1} of {STAGE_COUNT}"
                            >
                                {#each MASTERY_STAGES as _stage, i}
                                    <span class="seg" class:filled={i <= topic.stage}
                                    ></span>
                                {/each}
                            </span>
                            <span class="stage-label">{topic.stageLabel}</span>
                        </li>
                    {/each}
                </ul>
            </div>
        {/each}
    </div>
{/if}

<style lang="scss">
    // The signature of the page: four rungs left to right joined by one climb
    // line, each carrying a count. Boldness lives here and in the per-topic track;
    // the accent marks a rung's node only when a topic actually sits on it.
    .ladder {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 0.75rem;
        margin: 0;
        padding: 0;
        list-style: none;
    }
    .rung {
        position: relative;
        padding-top: 0.95rem;
        // The climb line, drawn behind the nodes and continued across the column
        // gap so the four rungs read as one axis.
        &::before {
            content: "";
            position: absolute;
            top: 0.34rem;
            inset-inline-start: 0;
            inset-inline-end: -0.75rem;
            height: 2px;
            background: var(--border-subtle);
        }
        // The axis ends at the last node instead of trailing past it: the
        // previous rung's line already bridges the gap up to this node.
        &:last-child::before {
            inset-inline-end: calc(100% - 0.7rem);
        }
        // A node on the line for each rung; accent only when occupied.
        &::after {
            content: "";
            position: absolute;
            top: 0;
            inset-inline-start: 0;
            width: 0.7rem;
            height: 0.7rem;
            border-radius: 50%;
            background: var(--canvas);
            border: 2px solid var(--border-subtle);
        }
        &.occupied::after {
            background: var(--sr-accent, var(--accent-card));
            border-color: var(--sr-accent, var(--accent-card));
        }
    }
    .rung-count {
        display: block;
        font-size: 1.6rem;
        font-weight: 660;
        line-height: 1;
        font-variant-numeric: tabular-nums;
    }
    .rung-label {
        display: block;
        margin-top: 0.3rem;
        font-size: 0.85rem;
        font-weight: 600;
    }
    .rung-blurb {
        display: block;
        margin-top: 0.15rem;
        max-width: 18ch;
        font-size: 0.75rem;
        line-height: 1.35;
        color: var(--fg-subtle);
    }

    .groups {
        margin-top: 2.25rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }
    .group-heading {
        margin: 0 0 0.5rem;
        font-size: 0.7rem;
        font-weight: 700;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--fg-subtle);
        border: none;
    }
    .rows {
        margin: 0;
        padding: 0;
        list-style: none;
    }
    .row {
        display: grid;
        grid-template-columns: minmax(8rem, 1fr) auto minmax(5.5rem, auto);
        align-items: center;
        gap: 0.75rem 1rem;
        padding: 0.55rem 0;
        border-top: 1px solid var(--border-subtle);
        &:first-child {
            border-top: none;
        }
    }
    .topic-label {
        font-size: 0.95rem;
        overflow-wrap: anywhere;
    }

    // The per-topic track: four segments filled left to right up to the topic's
    // current rung, so a glance down the column shows who is furthest along.
    .track {
        display: inline-flex;
        gap: 3px;
    }
    .seg {
        width: 1.5rem;
        height: 0.42rem;
        border-radius: 2px;
        background: var(--border-subtle);
        &.filled {
            background: var(--sr-accent, var(--accent-card));
        }
    }
    .stage-label {
        font-size: 0.85rem;
        color: var(--fg-subtle);
        text-align: end;
        font-variant-numeric: tabular-nums;
    }

    .state {
        .headline {
            margin: 0 0 0.4rem;
            font-size: 1rem;
            font-weight: 600;
        }
        .detail {
            margin: 0;
            max-width: 46ch;
            font-size: 0.9rem;
            line-height: 1.5;
            color: var(--fg-subtle);
        }
    }

    @media (max-width: 34rem) {
        .ladder {
            grid-template-columns: repeat(2, 1fr);
            gap: 1rem 0.75rem;
        }
        // Two per row: cap every line at its own cell so it never bleeds past a
        // wrap.
        .rung::before {
            inset-inline-end: 0;
        }
        .row {
            grid-template-columns: 1fr auto;
        }
        // On narrow screens the stage name carries the state; the track would wrap
        // awkwardly, so drop it.
        .track {
            display: none;
        }
    }
</style>
