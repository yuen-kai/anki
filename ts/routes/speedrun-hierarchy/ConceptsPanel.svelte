<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import ConceptModal from "./ConceptModal.svelte";
    import { type Concept, getTreeContext, newConcept, type Node } from "./lib";

    export let node: Node;

    const ctx = getTreeContext();

    let editing: Concept | null = null;

    function add(): void {
        const concept = newConcept();
        node.concepts = [...node.concepts, concept];
        ctx.change();
        editing = concept;
    }

    function remove(id: string): void {
        node.concepts = node.concepts.filter((concept) => concept.id !== id);
        ctx.change();
        if (editing?.id === id) {
            editing = null;
        }
    }

    function openConcept(concept: Concept): void {
        editing = concept;
    }

    // The modal edits the same concept object in place, so re-reading the list
    // (new array reference) reflects title/problem-count changes, then we save.
    function onModalChange(): void {
        node.concepts = [...node.concepts];
        ctx.change();
    }

    function close(): void {
        editing = null;
    }

    $: topic = node.title.trim();
</script>

<div class="panel">
    <div class="head">
        <h2 class="title">Concepts</h2>
        <button class="add" on:click={add}>Add concept</button>
    </div>
    <p class="topic">{topic || "Untitled topic"}</p>

    {#if node.concepts.length === 0}
        <p class="empty">No concepts yet.</p>
    {:else}
        <ul class="list">
            {#each node.concepts as concept (concept.id)}
                <li class="item">
                    <button class="open" on:click={() => openConcept(concept)}>
                        <span class="name">
                            {concept.title || "Untitled concept"}
                        </span>
                        <span class="meta">
                            {concept.problems.length}
                            {concept.problems.length === 1 ? "problem" : "problems"}
                        </span>
                    </button>
                    <button
                        class="del"
                        on:click={() => remove(concept.id)}
                        aria-label="Delete concept"
                    >
                        ×
                    </button>
                </li>
            {/each}
        </ul>
    {/if}
</div>

{#if editing}
    <ConceptModal concept={editing} onChange={onModalChange} onClose={close} />
{/if}

<style lang="scss">
    .panel {
        display: flex;
        flex-direction: column;
    }
    .head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
    }
    .title {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 660;
        letter-spacing: -0.01em;
    }
    .topic {
        margin: 0.15rem 0 1rem;
        font-size: 0.85rem;
        color: var(--sr-ink-3);
    }

    .add {
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        border-radius: 6px;
        padding: 0.4rem 0.7rem;
        font: inherit;
        font-size: 0.8rem;
        font-weight: 560;
        cursor: pointer;
        white-space: nowrap;
    }
    .add:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }

    .empty {
        margin: 0;
        color: var(--sr-ink-3);
        font-size: 0.9rem;
    }

    .list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    .item {
        display: flex;
        align-items: stretch;
        gap: 0.4rem;
    }
    .open {
        flex: 1 1 auto;
        min-width: 0;
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 0.75rem;
        border: 1px solid var(--sr-line);
        background: var(--sr-panel-2);
        border-radius: 6px;
        padding: 0.55rem 0.7rem;
        font: inherit;
        color: var(--sr-ink);
        text-align: left;
        cursor: pointer;
    }
    .open:hover {
        border-color: var(--sr-ink-3);
    }
    .name {
        font-weight: 560;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .meta {
        flex-shrink: 0;
        font-family: var(--sr-mono);
        font-size: 0.72rem;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }
    .del {
        flex-shrink: 0;
        width: 2rem;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel);
        color: var(--sr-ink-3);
        border-radius: 6px;
        font-size: 1.05rem;
        line-height: 1;
        cursor: pointer;
        opacity: 0;
        transition:
            opacity 0.12s ease,
            color 0.12s ease,
            border-color 0.12s ease;
    }
    .item:hover .del,
    .item:focus-within .del {
        opacity: 1;
    }
    .del:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }
</style>
