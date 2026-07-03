<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import StageGlyph from "../speedrun-dashboard/StageGlyph.svelte";
    import type { ConceptTreeNode } from "./lib";

    // A single topic (leaf) row: its title and the shared 4-segment mastery
    // glyph, filled up to the current stage in that stage's one colour (canvas
    // 8x4 md size).
    export let node: ConceptTreeNode;
</script>

<div class="leaf">
    <span class="leaf-name">{node.title}</span>
    <StageGlyph stage={node.stage} size="md" label={node.stageLabel} />
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .leaf {
        width: 250px;
        max-width: 100%;
        box-sizing: border-box;
        @include syn.tree-leaf;
    }
    .leaf-name {
        flex: 1;
        min-width: 0;
        font-size: 12px;
        font-weight: 600;
        overflow-wrap: anywhere;
    }
</style>
