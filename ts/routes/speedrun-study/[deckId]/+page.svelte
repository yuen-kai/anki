<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { overviewAction, showDecks, startStudy } from "../lib";
    import StudyOverview from "../StudyOverview.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;
</script>

<StudyOverview
    summary={data.summary}
    summaryError={data.summaryError}
    memory={data.memory}
    memoryError={data.memoryError}
    performance={data.performance}
    performanceError={data.performanceError}
    readiness={data.readiness}
    readinessError={data.readinessError}
    tree={data.tree}
    treeError={data.treeError}
    subjects={data.subjects}
    breakdownError={data.breakdownError}
    filtered={data.filtered}
    onStart={() => startStudy(data.deckId)}
    onBack={showDecks}
    onAction={overviewAction}
/>
