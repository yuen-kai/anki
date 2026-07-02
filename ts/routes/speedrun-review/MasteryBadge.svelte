<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { stageLabel, STAGE_TOTAL, stageRank } from "./lib";

    export let state: string;

    $: rank = stageRank(state);
    $: label = stageLabel(state);
    // One bar per rung; bars up to and including the current rung are lit, so the
    // meter reads as a rising ladder rather than a generic progress bar.
    $: rungs = Array.from({ length: STAGE_TOTAL }, (_, i) => i <= rank);
</script>

<span class="badge" title={`Mastery: ${label}`}>
    <span class="meter" aria-hidden="true">
        {#each rungs as lit, i (i)}
            <span class="rung" class:lit style={`--h: ${40 + i * 20}%`}></span>
        {/each}
    </span>
    <span class="label">{label}</span>
</span>

<style lang="scss">
    .badge {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.24rem 0.6rem 0.24rem 0.45rem;
        border: 1px solid var(--sr-line-2);
        border-radius: 999px;
        background: var(--sr-panel-2);
    }
    .meter {
        display: inline-flex;
        align-items: flex-end;
        gap: 2px;
        height: 0.95rem;
    }
    .rung {
        width: 3px;
        height: var(--h);
        border-radius: 1px;
        background: var(--sr-line-2);
    }
    .rung.lit {
        background: var(--sr-signal);
    }
    .label {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--sr-ink-2);
    }
</style>
