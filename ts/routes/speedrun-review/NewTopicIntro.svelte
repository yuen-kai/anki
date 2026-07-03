<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The new-topic intro: a centred tree that lights the path from the deck root
down to the newly unlocked leaf, then the leaf's title and a Begin CTA. Built
from the real authored hierarchy, so it generalises to any shape.
-->
<script lang="ts">
    import { type Node, pathToNode } from "./lib";
    import TopicTreeNode from "./TopicTreeNode.svelte";

    export let root: Node;
    export let topicNodeId: string;
    export let onStart: () => void;

    // [root, ...ancestors, target]; empty if the id isn't in the tree.
    $: path = pathToNode(root, topicNodeId) ?? [];
    $: target = path.length ? path[path.length - 1] : null;
    $: title = target?.title.trim() || "New topic";
    $: concepts = target ? countConcepts(target) : 0;

    function countConcepts(node: Node): number {
        return (
            node.concepts.length +
            node.children.reduce((n, c) => n + countConcepts(c), 0)
        );
    }

    function kindAt(index: number): "root" | "middle" | "target" {
        if (index === 0) {
            return "root";
        }
        return index === path.length - 1 ? "target" : "middle";
    }

    function onKey(event: KeyboardEvent): void {
        if (event.key === "Enter") {
            event.preventDefault();
            onStart();
        }
    }
</script>

<svelte:window on:keydown={onKey} />

<section class="card">
    <div class="glow" aria-hidden="true"></div>
    <div class="body">
        <p class="eyebrow">New topic unlocked</p>

        {#if path.length}
            <div class="tree">
                {#each path as node, index (node.id)}
                    <TopicTreeNode
                        {node}
                        kind={kindAt(index)}
                        siblings={index === 0 ? [] : path[index - 1].children}
                        connector={index > 0}
                    />
                {/each}
            </div>
        {/if}

        <div class="foot">
            <h1 class="title">{title}</h1>
            <p class="sub">
                {concepts}
                {concepts === 1 ? "concept" : "concepts"} · begins at the Learn stage
            </p>
            <button class="begin" type="button" on:click={onStart}>Begin</button>
        </div>
    </div>
</section>

<style lang="scss">
    .card {
        position: relative;
        overflow: hidden;
        box-sizing: border-box;
        width: 100%;
        max-width: 42rem;
        margin: 0 auto;
        padding: 24px 28px 28px;
        background: var(--sr-panel);
        border-radius: var(--sr-radius-card);
        box-shadow: var(--sr-shadow-card);
        font-family: var(--sr-sans);
    }
    .glow {
        position: absolute;
        inset: 0;
        pointer-events: none;
        background: radial-gradient(
            circle at 50% -10%,
            color-mix(in srgb, var(--sr-signal) 10%, transparent),
            transparent 45%
        );
    }
    .body {
        position: relative;
        text-align: center;
    }

    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.22em;
        text-transform: uppercase;
        color: var(--sr-signal);
    }

    .tree {
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-top: 22px;
    }

    .foot {
        margin-top: 18px;
    }
    .title {
        margin: 0;
        font-size: 20px;
        font-weight: 700;
        color: var(--sr-ink);
    }
    .sub {
        margin: 3px 0 0;
        font-size: 12px;
        color: var(--sr-ink-3);
    }
    .begin {
        appearance: none;
        cursor: pointer;
        margin-top: 14px;
        padding: 12px 36px;
        border: 0;
        border-radius: 13px;
        background: var(--sr-signal);
        // On-coral label stays literally white in both themes; --sr-white is a
        // surface token that flips dark at night.
        color: #fff;
        font-family: var(--sr-sans);
        font-size: 14px;
        font-weight: 700;
        box-shadow: var(--sr-shadow-signal-lg);
    }
    .begin:hover {
        filter: brightness(1.04);
    }
</style>
