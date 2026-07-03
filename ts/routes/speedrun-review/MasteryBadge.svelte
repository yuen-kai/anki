<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The session stage tag: a mono, letter-spaced stage name (LEARN / PRACTICE /
GUIDED / SOLO) tinted in that stage's soft fill, with the 4-segment mastery
glyph inline. Sits in the header above each session card so the learner always
sees which rung of the ladder this card is on.
-->
<script lang="ts">
    import StageGlyph from "../speedrun-dashboard/StageGlyph.svelte";
    import { stageLabel, stageRank } from "./lib";

    // The engine state name (learning / practicing / hierarchy / mastering).
    export let state: string;

    // learning->learn, practicing->practice, hierarchy->guided, mastering->solo:
    // the ladder-position names that key the stage tint tokens.
    const NAMES = ["learn", "practice", "guided", "solo"] as const;

    $: rank = stageRank(state);
    $: name = NAMES[rank] ?? NAMES[0];
    $: label = stageLabel(state);
</script>

<span class="tag tag--{name}">
    <span class="name">{label.toUpperCase()}</span>
    <StageGlyph stage={rank} size="sm" label={`${label}, stage ${rank + 1} of 4`} />
</span>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .tag {
        @include syn.stage-tag;
    }
    .tag--learn {
        @include syn.stage-tint(learn);
    }
    .tag--practice {
        @include syn.stage-tint(practice);
    }
    .tag--guided {
        @include syn.stage-tint(guided);
    }
    .tag--solo {
        @include syn.stage-tint(solo);
    }
    .name {
        line-height: 1;
    }
</style>
