<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import SpeedrunDashboard from "../SpeedrunDashboard.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;
</script>

<SpeedrunDashboard
    memory={data.memory}
    memoryError={data.memoryError}
    performance={data.performance}
    performanceError={data.performanceError}
    readiness={data.readiness}
    readinessError={data.readinessError}
    topics={data.topics}
    topicsError={data.topicsError}
/>
