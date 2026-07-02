<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import {
        buildTopicsView,
        deferredEnvelope,
        formatCoverage,
        type ScoreEnvelope,
        type TopicsView,
    } from "./lib";
    import ScoreTile from "./ScoreTile.svelte";
    import TopicsProgress from "./TopicsProgress.svelte";

    export let memory: ScoreEnvelope | null;
    export let memoryError: string | null = null;
    export let performance: ScoreEnvelope | null = null;
    export let performanceError: string | null = null;
    export let readiness: ScoreEnvelope | null = null;
    export let readinessError: string | null = null;
    export let topics: TopicsView = buildTopicsView(null);
    export let topicsError: string | null = null;

    // The masthead readout reflects the deck's collected evidence. Any score's
    // envelope carries it (they share coverage + graded-review counts), so take
    // the first one that exists, even if it is abstaining.
    $: head = memory ?? performance ?? readiness ?? null;
</script>

<div class="dashboard">
    <div class="inner">
        <header class="masthead">
            <p class="wordmark">
                Speedrun
                <span class="wordmark-sub">MCAT readiness</span>
            </p>
            {#if head}
                <dl class="readout">
                    <div>
                        <dt>Coverage</dt>
                        <dd>{formatCoverage(head.coveragePct)}</dd>
                    </div>
                    <div>
                        <dt>Reviews</dt>
                        <dd>{head.gradedReviews}</dd>
                    </div>
                </dl>
            {/if}
        </header>

        <!-- Scores lead: they answer the one question the learner opens for. -->
        <section class="section">
            <div class="sec-head">
                <h2 class="sec-title">Scores</h2>
                <p class="sec-note">
                    Three questions, three answers, never blended into one.
                </p>
            </div>
            <div class="instruments">
                <ScoreTile
                    name="Memory"
                    question="Will you recall a fact you were taught, right now?"
                    live={true}
                    envelope={memory ?? deferredEnvelope("")}
                    error={memoryError}
                />
                <ScoreTile
                    name="Performance"
                    question="Will you get a new, exam-style question right?"
                    live={true}
                    envelope={performance ?? deferredEnvelope("")}
                    error={performanceError}
                />
                <ScoreTile
                    name="Readiness"
                    question="What would you score on the 472 to 528 scale today?"
                    live={true}
                    envelope={readiness ?? deferredEnvelope("")}
                    error={readinessError}
                />
            </div>
        </section>

        <section class="section">
            <div class="sec-head">
                <h2 class="sec-title">Mastery</h2>
                <p class="sec-note">
                    Every topic climbs four stages, from first contact to working
                    problems unaided.
                </p>
            </div>
            <TopicsProgress view={topics} error={topicsError} />
        </section>
    </div>
</div>

<style lang="scss">
    .dashboard {
        // The "score instrument" identity, defined here so the tiles and the
        // topics view inherit one palette. Cool paper, charcoal ink, one marigold
        // signal; a mono readout label system alongside the reading sans.
        --sr-paper: #e7eaec;
        --sr-panel: #fcfcfc;
        --sr-panel-2: #f0f2f3;
        --sr-ink: #181b20;
        --sr-ink-2: #565c64;
        --sr-ink-3: #878d95;
        --sr-line: #d5d9dc;
        --sr-line-2: #c1c6cb;
        --sr-signal: #df8f0a;
        --sr-signal-ink: #3a2a05;
        --sr-signal-weak: #f4e6cb;
        --sr-signal-line: #e6c079;
        --sr-hatch: rgba(0, 0, 0, 0.014);
        --sr-sans:
            "Inter", ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI",
            Roboto, "Helvetica Neue", Arial, sans-serif;
        --sr-mono:
            ui-monospace, "SF Mono", "SFMono-Regular", Menlo, "Cascadia Code",
            "Roboto Mono", Consolas, monospace;

        box-sizing: border-box;
        min-height: 100%;
        padding: 2.25rem 1.75rem 3rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;
    }
    :global(.night-mode) .dashboard {
        --sr-paper: #131519;
        --sr-panel: #1b1e22;
        --sr-panel-2: #15171b;
        --sr-ink: #f0eee9;
        --sr-ink-2: #a6abb2;
        --sr-ink-3: #767c85;
        --sr-line: #2a2e34;
        --sr-line-2: #3a3f46;
        --sr-signal: #f2a93b;
        --sr-signal-ink: #1b1405;
        --sr-signal-weak: #2c2314;
        --sr-signal-line: #5a4620;
        --sr-hatch: rgba(255, 255, 255, 0.02);
    }

    .inner {
        max-width: 62rem;
        margin: 0 auto;
    }

    .masthead {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
        gap: 1.5rem;
        padding-bottom: 1rem;
        border-bottom: 2px solid var(--sr-ink);
    }
    .wordmark {
        margin: 0;
        display: flex;
        align-items: baseline;
        gap: 0.6rem;
        font-size: 1.4rem;
        font-weight: 720;
        letter-spacing: -0.02em;
    }
    .wordmark-sub {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .readout {
        display: flex;
        gap: 1.75rem;
        margin: 0;
    }
    .readout div {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        text-align: right;
    }
    .readout dt {
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .readout dd {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 640;
        font-variant-numeric: tabular-nums;
    }

    .section {
        margin-top: 2.75rem;
    }
    .sec-head {
        display: flex;
        align-items: baseline;
        gap: 0.9rem;
        flex-wrap: wrap;
        margin-bottom: 1.25rem;
    }
    .sec-title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 680;
        letter-spacing: -0.015em;
        line-height: 1.1;
        border: none;
    }
    .sec-note {
        margin: 0;
        font-size: 0.88rem;
        color: var(--sr-ink-2);
    }

    .instruments {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        overflow: hidden;
    }

    @media (max-width: 46rem) {
        .instruments {
            grid-template-columns: 1fr;
        }
        // Stacked, the divider becomes a horizontal rule between rows.
        :global(.instruments) > :global(.ins) {
            border-left: none !important;
            border-top: 1px solid var(--sr-line);
        }
        :global(.instruments) > :global(.ins:first-child) {
            border-top: none;
        }
    }
</style>
