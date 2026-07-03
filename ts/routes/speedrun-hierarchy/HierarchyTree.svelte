<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import HierarchyNode from "./HierarchyNode.svelte";
    import type { Node } from "./lib";

    export let root: Node;
    // Bumped by the editor on every edit; forwarded so each node can recompute
    // its rolled-up counts through in-place mutations. Defaults to 0 for callers
    // (e.g. the demo) that render a static tree.
    export let rev = 0;
</script>

<ul class="tree">
    <HierarchyNode node={root} ancestors={[]} depth={0} {rev} onRemove={null} />
</ul>

<style lang="scss">
    .tree {
        list-style: none;
        margin: 0;
        padding: 0;
    }
</style>
