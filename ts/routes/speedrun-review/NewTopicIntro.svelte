<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { type Node, pathToNode } from "./lib";
    import TopicTreeNode from "./TopicTreeNode.svelte";

    export let root: Node;
    export let topicNodeId: string;
    export let onStart: () => void;

    $: path = pathToNode(root, topicNodeId) ?? [];
    $: pathOrder = new Map(path.map((node, i) => [node.id, i] as const));
    $: target = path.length ? path[path.length - 1] : null;

    function onKey(event: KeyboardEvent): void {
        if (event.key === "Enter") {
            event.preventDefault();
            onStart();
        }
    }
</script>

<svelte:window on:keydown={onKey} />

<div class="intro">
    <div class="lead">
        <p class="eyebrow">New topic</p>
        <h1 class="topic">{target?.title.trim() || "New topic"}</h1>
    </div>

    <div class="map">
        <p class="map-label">In your hierarchy</p>
        <ul class="tree">
            <TopicTreeNode node={root} {pathOrder} />
        </ul>
    </div>

    <button class="btn primary" on:click={onStart}>Start learning</button>
</div>

<style lang="scss">
    .intro {
        width: 100%;
        max-width: 40rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.75rem;
        text-align: center;
    }
    .lead {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
        animation: sr-rise 400ms ease both;
    }
    @keyframes sr-rise {
        from {
            opacity: 0;
            transform: translateY(6px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    @media (prefers-reduced-motion: reduce) {
        .lead {
            animation: none;
        }
    }
    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 12px;
        font-weight: 600;
        letter-spacing: 0.2em;
        text-transform: uppercase;
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .eyebrow {
        color: var(--sr-signal);
    }
    .topic {
        margin: 0;
        font-size: 2.1rem;
        font-weight: 720;
        letter-spacing: -0.02em;
        line-height: 1.1;
    }

    .map {
        width: 100%;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 12px;
        padding: 1.1rem 1.25rem 1.25rem;
        text-align: left;
    }
    .map-label {
        margin: 0 0 0.6rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .tree {
        list-style: none;
        margin: 0;
        padding: 0;
    }

    .btn {
        appearance: none;
        border: 1px solid transparent;
        border-radius: 8px;
        padding: 0.7rem 1.6rem;
        font: inherit;
        font-weight: 600;
        cursor: pointer;
    }
    .btn.primary {
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
    }
    .btn.primary:hover {
        filter: brightness(1.04);
    }

    @media (max-width: 34rem) {
        .topic {
            font-size: 1.6rem;
        }
    }
</style>
