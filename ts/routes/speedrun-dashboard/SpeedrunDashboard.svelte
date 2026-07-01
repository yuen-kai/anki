<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import {
        buildTopicsView,
        deferredEnvelope,
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
</script>

<div class="dashboard">
    <div class="inner">
        <!-- Topics leads: it has real data from the first review, while the three
             scores abstain until enough graded reviews exist. -->
        <section class="section">
            <header class="section-head">
                <h2 class="section-title">Topics</h2>
                <p class="section-lead">
                    Where each topic sits on the four-stage mastery ladder.
                </p>
            </header>
            <TopicsProgress view={topics} error={topicsError} />
        </section>

        <section class="section section--scores">
            <header class="section-head">
                <h2 class="section-title">Scores</h2>
                <p class="section-lead">
                    Three separate scores, never blended into one number.
                </p>
            </header>
            <div class="tiles">
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
    </div>
</div>

<style lang="scss">
    .dashboard {
        box-sizing: border-box;
        min-height: 100%;
        padding: 1.75rem 1.5rem 2.5rem;
        background: var(--canvas);
        color: var(--fg);
        font-size: var(--font-size);
        // Match the study cards' identity: a system-first sans (renders the same
        // offline on desktop and mobile) with the same tight tracking, so the
        // dashboard and the cards read as one product.
        font-family: "Inter", ui-sans-serif, -apple-system, BlinkMacSystemFont,
            "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
        letter-spacing: -0.003em;

        // Speedrun's one product accent, the same teal the study cards use, so the
        // dashboard, the topics ladder, and the cards read as one product instead
        // of inheriting Anki's default. Children inherit it.
        --sr-accent: #0e7c66;
    }
    :global(.night-mode) .dashboard {
        --sr-accent: #4fd1ac;
    }
    .inner {
        max-width: 64rem;
        margin: 0 auto;
    }

    // Two readouts, one rhythm. Topics leads because it has data from the first
    // review; the scores follow. Open space alone sets them apart: a rule here
    // would only echo the topic row separators.
    .section--scores {
        margin-top: 3rem;
    }

    .section-head {
        margin-bottom: 1.5rem;
    }
    .section-title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 650;
        line-height: 1.15;
        letter-spacing: -0.012em;
        border: none;
    }
    .section-lead {
        margin: 0.4rem 0 0;
        max-width: 54ch;
        font-size: 0.95rem;
        line-height: 1.5;
        color: var(--fg-subtle);
    }

    .tiles {
        display: grid;
        gap: 1rem;
        grid-template-columns: repeat(auto-fit, minmax(15rem, 1fr));
        align-items: start;
    }
</style>
