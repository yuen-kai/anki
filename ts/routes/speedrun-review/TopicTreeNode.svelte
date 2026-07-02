<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

A read-only render of the authored hierarchy. Nodes on the path to the new topic
light up in order, so the highlight climbs down the tree to the target leaf.
-->
<script lang="ts">
    import type { Node } from "./lib";

    export let node: Node;
    // conceptId/nodeId -> its position on the highlighted path (0 = root).
    export let pathOrder: Map<string, number>;
    export let depth = 0;

    const STEP_MS = 200;

    $: order = pathOrder.get(node.id);
    $: onPath = order !== undefined;
    $: leaf = node.children.length === 0;
</script>

<li class="node" class:child={depth > 0}>
    <div
        class="row"
        class:on-path={onPath}
        class:target={onPath && leaf}
        style={onPath ? `--delay: ${(order ?? 0) * STEP_MS}ms` : ""}
    >
        <span class="name" class:root={depth === 0}>
            {node.title.trim() || "Untitled"}
        </span>
        {#if leaf && node.concepts.length}
            <span class="count">{node.concepts.length}</span>
        {/if}
    </div>

    {#if node.children.length}
        <ul class="children">
            {#each node.children as child (child.id)}
                <svelte:self node={child} {pathOrder} depth={depth + 1} />
            {/each}
        </ul>
    {/if}
</li>

<style lang="scss">
    .node {
        list-style: none;
    }
    .row {
        position: relative;
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        min-height: 2rem;
        padding: 0.2rem 0.6rem;
        border-radius: 6px;
        box-shadow: inset 0 0 0 1px transparent;
    }
    .row.on-path {
        background: var(--sr-signal-weak);
        box-shadow: inset 0 0 0 1px var(--sr-signal-line);
        animation: sr-light 360ms ease both;
        animation-delay: var(--delay, 0ms);
    }
    @keyframes sr-light {
        from {
            background: transparent;
            box-shadow: inset 0 0 0 1px transparent;
        }
        to {
            background: var(--sr-signal-weak);
            box-shadow: inset 0 0 0 1px var(--sr-signal-line);
        }
    }
    @media (prefers-reduced-motion: reduce) {
        .row.on-path {
            animation: none;
        }
    }

    .name {
        color: var(--sr-ink-2);
        font-size: 0.95rem;
    }
    .row.on-path .name {
        color: var(--sr-signal-ink);
        font-weight: 560;
    }
    :global(.night-mode) .row.on-path .name {
        color: var(--sr-signal);
    }
    .name.root {
        font-weight: 680;
        font-size: 1.05rem;
        color: var(--sr-ink);
    }
    .row.target .name {
        font-weight: 680;
    }
    .count {
        font-family: var(--sr-mono);
        font-size: 0.68rem;
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink-2);
        background: var(--sr-panel);
        border: 1px solid var(--sr-line-2);
        border-radius: 999px;
        padding: 0.02rem 0.4rem;
    }

    // Blueprint connectors, matching the authoring tree's vocabulary.
    .children {
        list-style: none;
        margin: 0;
        padding: 0;
    }
    .node.child {
        position: relative;
        padding-left: 1.4rem;
    }
    .node.child::before {
        content: "";
        position: absolute;
        left: 0;
        top: 1rem;
        width: 1rem;
        border-top: 1.5px solid var(--sr-line-2);
    }
    .node.child::after {
        content: "";
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        border-left: 1.5px solid var(--sr-line-2);
    }
    .node.child:last-child::after {
        bottom: auto;
        height: 1rem;
    }
</style>
