<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { MASTERY_STAGES, STAGE_COUNT, type TopicsView } from "./lib";

    export let view: TopicsView;
    export let error: string | null = null;
</script>

<section class="topics">
    <header class="topics-header">
        <h2>Topics</h2>
        <p class="lead">Where each topic sits on the four-stage mastery ladder.</p>
    </header>

    {#if error}
        <div class="state">
            <p class="headline">Couldn't load topic progress</p>
            <p class="detail">{error}</p>
        </div>
    {:else if view.total === 0}
        <div class="state">
            <p class="headline">No topics tracked yet</p>
            <p class="detail">
                Study this deck to start moving its topics up the ladder, from
                learning to mastering.
            </p>
        </div>
    {:else}
        <!-- The ladder is the map: every topic climbs these four rungs in order,
             so the summary counts how many sit on each. -->
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
                                        <span
                                            class="seg"
                                            class:filled={i <= topic.stage}
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
</section>

<style lang="scss">
    .topics {
        margin-top: 2.5rem;
    }
    .topics-header {
        h2 {
            margin: 0;
            font-size: 1.25rem;
            font-weight: 660;
            border: none;
        }
        .lead {
            margin: 0.3rem 0 0;
            font-size: 0.9rem;
            color: var(--fg-subtle);
        }
    }

    // The ladder summary: four rungs left to right, joined by a hairline that
    // reads as the climb. Boldness lives here and in the per-topic track; the
    // accent marks a rung only when a topic actually sits on it.
    .ladder {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 0.75rem;
        margin: 1.25rem 0 0;
        padding: 0;
        list-style: none;
    }
    .rung {
        position: relative;
        padding-top: 0.9rem;
        // The connecting climb line, drawn behind the rung markers.
        &::before {
            content: "";
            position: absolute;
            top: 0.32rem;
            left: 0;
            right: 0;
            height: 2px;
            background: var(--border-subtle);
        }
        // A filled node on the line for each rung; accent only when occupied.
        &::after {
            content: "";
            position: absolute;
            top: 0;
            inset-inline-start: 0;
            width: 0.66rem;
            height: 0.66rem;
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
        font-size: 1.5rem;
        font-weight: 660;
        line-height: 1;
        font-variant-numeric: tabular-nums;
    }
    .rung-label {
        display: block;
        margin-top: 0.2rem;
        font-size: 0.85rem;
        font-weight: 550;
    }
    .rung-blurb {
        display: block;
        margin-top: 0.15rem;
        font-size: 0.75rem;
        line-height: 1.35;
        color: var(--fg-subtle);
    }

    .groups {
        margin-top: 2rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }
    .group-heading {
        margin: 0 0 0.5rem;
        font-size: 0.8rem;
        font-weight: 600;
        letter-spacing: 0.02em;
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
        padding: 0.5rem 0;
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
        margin-top: 1.25rem;
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
        .row {
            grid-template-columns: 1fr auto;
        }
        // On narrow screens the explicit stage name carries the state; the
        // decorative track would wrap awkwardly, so drop it.
        .track {
            display: none;
        }
    }
</style>
