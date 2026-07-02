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
            Study this deck to start moving its topics up the ladder, from learning to
            mastering.
        </p>
    </div>
{:else}
    <!-- The overview: one bar split by how many topics sit on each stage, with a
         legend beneath. The last stage (mastering) carries the signal. -->
    <div class="dist">
        <div class="bar" aria-hidden="true">
            {#each MASTERY_STAGES as _stage, i}
                {#if view.distribution[i] > 0}
                    <span class="seg s{i}" style="flex:{view.distribution[i]}"></span>
                {/if}
            {/each}
        </div>
        <dl class="legend">
            {#each MASTERY_STAGES as stage, i}
                <div class="leg">
                    <dt>
                        <span class="dot s{i}"></span>
                        {stage.label}
                    </dt>
                    <dd class="n">{view.distribution[i]}</dd>
                    <dd class="b">{stage.blurb}</dd>
                </div>
            {/each}
        </dl>
    </div>

    <div class="groups">
        {#each view.groups as group}
            <div class="group">
                <h3 class="group-heading">{group.heading}</h3>
                <ul class="rows">
                    {#each group.topics as topic}
                        <li class="row">
                            <span class="t-name">{topic.label}</span>
                            <span
                                class="meter"
                                role="img"
                                aria-label="{topic.stageLabel}, stage {topic.stage +
                                    1} of {STAGE_COUNT}"
                            >
                                {#each MASTERY_STAGES as _stage, i}
                                    <i
                                        class:on={i <= topic.stage}
                                        class:top={i === topic.stage &&
                                            topic.stage === STAGE_COUNT - 1}
                                    ></i>
                                {/each}
                            </span>
                            <span
                                class="t-stage"
                                class:done={topic.stage === STAGE_COUNT - 1}
                            >
                                {topic.stageLabel}
                            </span>
                        </li>
                    {/each}
                </ul>
            </div>
        {/each}
    </div>
{/if}

<style lang="scss">
    .dist {
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        padding: 1.25rem 1.35rem 1.35rem;
    }
    .bar {
        display: flex;
        gap: 2px;
        height: 14px;
        border-radius: 2px;
        overflow: hidden;
    }
    .seg {
        height: 100%;
    }
    // Stage tints climb from quiet to the signal at "mastering".
    .s0 {
        background: var(--sr-line-2);
    }
    .s1 {
        background: var(--sr-ink-3);
    }
    .s2 {
        background: var(--sr-ink-2);
    }
    .s3 {
        background: var(--sr-signal);
    }

    .legend {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 0.75rem 1rem;
        margin: 1rem 0 0;
    }
    .leg dt {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .dot {
        width: 8px;
        height: 8px;
        border-radius: 1px;
        flex: 0 0 auto;
    }
    .leg .n {
        margin: 0.3rem 0 0;
        font-size: 1.5rem;
        font-weight: 680;
        line-height: 1;
        font-variant-numeric: tabular-nums;
    }
    .leg .b {
        margin: 0.15rem 0 0;
        font-size: 0.74rem;
        line-height: 1.3;
        color: var(--sr-ink-2);
    }

    .groups {
        margin-top: 1.9rem;
        display: flex;
        flex-direction: column;
        gap: 1.4rem;
    }
    .group-heading {
        margin: 0 0 0.35rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        border: none;
    }
    .rows {
        margin: 0;
        padding: 0;
        list-style: none;
    }
    .row {
        display: grid;
        grid-template-columns: minmax(0, 1fr) auto auto;
        align-items: center;
        gap: 1rem;
        padding: 0.6rem 0;
        border-top: 1px solid var(--sr-line);
    }
    .row:first-child {
        border-top: none;
    }
    .t-name {
        font-size: 0.96rem;
        font-weight: 500;
        overflow-wrap: anywhere;
    }

    // The per-topic meter: four segments filled to the topic's stage; ink for
    // progress, the signal only on a fully-mastered topic.
    .meter {
        display: inline-flex;
        gap: 3px;
    }
    .meter i {
        width: 30px;
        height: 7px;
        border-radius: 1px;
        background: var(--sr-line-2);
    }
    .meter i.on {
        background: var(--sr-ink);
    }
    .meter i.top {
        background: var(--sr-signal);
    }

    .t-stage {
        min-width: 5.5rem;
        text-align: end;
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .t-stage.done {
        color: var(--sr-signal);
    }

    .state .headline {
        margin: 0 0 0.4rem;
        font-size: 1rem;
        font-weight: 600;
    }
    .state .detail {
        margin: 0;
        max-width: 46ch;
        font-size: 0.9rem;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }

    @media (max-width: 34rem) {
        .legend {
            grid-template-columns: repeat(2, 1fr);
        }
        .row {
            grid-template-columns: 1fr auto;
        }
        // On narrow screens the mono stage label carries the state; the meter
        // would crowd, so drop it.
        .meter {
            display: none;
        }
    }
</style>
