<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { buildTopicsView, deferredEnvelope, type ScoreEnvelope, type TopicsView } from "./lib";
    import ScoreTile from "./ScoreTile.svelte";
    import TopicsProgress from "./TopicsProgress.svelte";

    export let memory: ScoreEnvelope | null;
    export let memoryError: string | null = null;
    export let topics: TopicsView = buildTopicsView(null);
    export let topicsError: string | null = null;

    // Designed but not built yet (PRD §6): each says when it will exist and why
    // it depends on the one before it.
    const performance = deferredEnvelope(
        "The performance score arrives in the Friday milestone, once a model can grade new, exam-style questions.",
    );
    const readiness = deferredEnvelope(
        "The readiness score arrives in the Friday milestone. It maps performance onto the 472 to 528 scale, so it follows performance.",
    );
</script>

<div class="dashboard">
    <div class="inner">
        <header class="dashboard-header">
            <h1>Scores</h1>
            <p class="principle">
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
                live={false}
                envelope={performance}
            />
            <ScoreTile
                name="Readiness"
                question="What would you score on the 472 to 528 scale today?"
                live={false}
                envelope={readiness}
            />
        </div>

        <TopicsProgress view={topics} error={topicsError} />
    </div>
</div>

<style lang="scss">
    .dashboard {
        box-sizing: border-box;
        min-height: 100%;
        padding: 1.5rem 1.5rem 2rem;
        background: var(--canvas);
        color: var(--fg);
        font-size: var(--font-size);
    }
    .inner {
        max-width: 70rem;
        margin: 0 auto;
    }
    .dashboard-header {
        h1 {
            margin: 0;
            font-size: 1.6rem;
            font-weight: 680;
        }
        .principle {
            margin: 0.35rem 0 0;
            font-size: 0.95rem;
            color: var(--fg-subtle);
        }
    }
    .tiles {
        display: grid;
        gap: 1rem;
        margin-top: 1.4rem;
        grid-template-columns: repeat(auto-fit, minmax(15rem, 1fr));
        align-items: start;
    }
</style>
