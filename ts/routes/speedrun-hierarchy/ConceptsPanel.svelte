<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import ConceptModal from "./ConceptModal.svelte";
    import { getTreeContext, newConcept, type Node } from "./lib";

    export let node: Node;
    // Forwarded from the editor so in-place edits (title / problem counts) refresh
    // the rows. Defaults to 0 for the demo, which renders a static list.
    export let rev = 0;
    // The open leaf's parent, shown as a breadcrumb. Optional (the demo omits it).
    export let parentName: string | null = null;
    // When the host drives selection (real editor), it passes the open concept id
    // and a callback and renders the editor itself. With no callback the panel is
    // self-contained: it tracks its own selection and shows the editor inline.
    export let selectedConceptId: string | null = null;
    export let onSelect: ((id: string | null) => void) | null = null;

    const ctx = getTreeContext();
    const delegated = onSelect !== null;

    let localSelected: string | null = null;
    // Reset the self-contained selection whenever the open leaf changes.
    let lastNodeId: string | null = null;
    $: if (node.id !== lastNodeId) {
        lastNodeId = node.id;
        localSelected = null;
    }

    function keep<T>(_rev: number, value: T): T {
        return value;
    }

    $: activeId = delegated ? selectedConceptId : localSelected;
    $: concepts = keep(rev, node.concepts);
    $: active = concepts.find((c) => c.id === activeId) ?? null;
    $: topic = node.title.trim();

    function select(id: string | null): void {
        if (onSelect) {
            onSelect(id);
        } else {
            localSelected = id;
        }
    }

    function add(): void {
        const concept = newConcept();
        node.concepts = [...node.concepts, concept];
        ctx.change();
        select(concept.id);
    }

    function remove(id: string): void {
        node.concepts = node.concepts.filter((concept) => concept.id !== id);
        ctx.change();
        if (activeId === id) {
            select(null);
        }
    }

    // The editor edits the same concept object in place; re-reading the list (new
    // array reference) reflects title / problem-count changes and marks the draft
    // dirty for the next explicit save.
    function onEditorChange(): void {
        node.concepts = [...node.concepts];
        ctx.change();
    }

    const pad = (i: number): string => String(i + 1).padStart(2, "0");
    const plural = (n: number, word: string): string =>
        `${n} ${word}${n === 1 ? "" : "s"}`;
</script>

<div class="concepts">
    <div class="head">
        <div class="crumb">
            {#if parentName}
                <span class="parent">{parentName} /</span>
            {/if}
            <span class="topic">{topic || "Untitled topic"}</span>
        </div>
        <p class="count">{plural(concepts.length, "concept")}</p>
    </div>

    <ul class="list">
        {#each concepts as concept, i (concept.id)}
            <li class="item" class:selected={concept.id === activeId}>
                <button class="open" on:click={() => select(concept.id)}>
                    <span class="no">{pad(i)}</span>
                    <span class="body">
                        <span class="name">
                            {concept.title.trim() || "Untitled concept"}
                        </span>
                        <span class="meta">
                            {plural(concept.problems.length, "practice problem")}
                        </span>
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

        <li class="affordance-item">
            <button class="affordance" on:click={add}>
                <span class="plus" aria-hidden="true">+</span>
                New concept
            </button>
        </li>
    </ul>

    {#if !delegated && active}
        <div class="inline-editor">
            <ConceptModal concept={active} onChange={onEditorChange} />
        </div>
    {/if}
</div>

<style lang="scss">
    .concepts {
        display: flex;
        flex-direction: column;
    }

    .head {
        margin-bottom: 16px;
    }
    .crumb {
        display: flex;
        align-items: baseline;
        gap: 10px;
        margin-bottom: 4px;
    }
    .parent {
        flex-shrink: 0;
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 500;
        color: var(--sr-ink-3);
    }
    .topic {
        font-size: 18px;
        font-weight: 700;
        letter-spacing: -0.01em;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .count {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 11px;
        color: var(--sr-faint);
    }

    .list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 7px;
    }
    .item {
        position: relative;
        display: flex;
        align-items: stretch;
        background: var(--sr-white);
        border: 1.5px solid var(--sr-line);
        border-radius: 11px;
    }
    .item.selected {
        border-color: var(--sr-signal);
    }

    // The whole row opens the concept; a numbered mono index leads it.
    .open {
        flex: 1 1 auto;
        min-width: 0;
        display: flex;
        align-items: center;
        gap: 12px;
        border: none;
        background: none;
        padding: 12px 14px;
        font: inherit;
        color: var(--sr-ink);
        text-align: left;
        cursor: pointer;
    }
    .no {
        flex-shrink: 0;
        width: 20px;
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 600;
        color: var(--sr-line-dashed);
    }
    .item.selected .no {
        color: var(--sr-signal);
    }
    .body {
        min-width: 0;
    }
    .name {
        display: block;
        font-size: 13px;
        font-weight: 600;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .meta {
        display: block;
        margin-top: 1px;
        font-family: var(--sr-mono);
        font-size: 10px;
        color: var(--sr-faint);
    }

    // Delete stays quiet until the row is hovered or focused.
    .del {
        flex-shrink: 0;
        width: 2.4rem;
        border: none;
        border-left: 1px solid var(--sr-line);
        background: none;
        color: var(--sr-ink-3);
        border-radius: 0 11px 11px 0;
        font-size: 1.05rem;
        line-height: 1;
        cursor: pointer;
        opacity: 0;
        transition: opacity 0.12s ease;
    }
    .item:hover .del,
    .item:focus-within .del {
        opacity: 1;
    }
    .del:hover {
        color: var(--sr-ink);
    }

    .affordance-item {
        list-style: none;
    }
    .affordance {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        width: fit-content;
        border: 1.5px dashed var(--sr-line-dashed);
        border-radius: 11px;
        background: none;
        padding: 11px 14px;
        font-family: var(--sr-sans);
        font-size: 12.5px;
        font-weight: 600;
        color: var(--sr-ink-3);
        cursor: pointer;
    }
    .affordance:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }
    .affordance .plus {
        font-size: 1em;
        line-height: 1;
    }

    // Self-contained mode (demo) shows the editor beneath the list.
    .inline-editor {
        margin-top: 18px;
        padding-top: 18px;
        border-top: 1px solid var(--sr-line);
    }

    @media (prefers-reduced-motion: reduce) {
        .del {
            transition: none;
        }
    }
</style>
