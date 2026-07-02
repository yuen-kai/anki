<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import HierarchyNode from "./HierarchyNode.svelte";
    import type { Node } from "./lib";

    export let root: Node;
</script>

<ul class="tree">
    <HierarchyNode node={root} ancestors={[]} depth={0} onRemove={null} />
</ul>

<style lang="scss">
    .tree {
        list-style: none;
        margin: 0;
        padding: 0;
    }
</style>
