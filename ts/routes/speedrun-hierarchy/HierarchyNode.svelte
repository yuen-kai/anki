<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { tick } from "svelte";

    import { getTreeContext, newNode, type Node } from "./lib";

    export let node: Node;
    export let ancestors: string[] = [];
    export let depth = 0;
    export let onRemove: (() => void) | null = null;
    // A monotonic revision from the editor. Child nodes mutate the shared tree in
    // place, so an edit deep in the branch does not change any ancestor's object
    // identity; threading `rev` down forces the rolled-up counts below to recompute.
    export let rev = 0;

    const ctx = getTreeContext();
    const { selectedId, pulseState } = ctx;

    const STAGGER_MS = 70;

    $: isRoot = depth === 0;
    $: hasChildren = node.children.length > 0;
    $: leaf = !isRoot && !hasChildren;
    $: selected = $selectedId === node.id;
    // The one coral trigger for a branch: it is the direct parent of the open
    // leaf. Selection only ever targets a leaf, so exactly one branch (the open
    // leaf's parent) can be active at a time.
    $: isSelectedParent = !isRoot && node.children.some((c) => c.id === $selectedId);
    $: childAncestors = [...ancestors, node.id];
    $: pulseIndex = $pulseState ? $pulseState.ids.indexOf(node.id) : -1;
    // `rev` is only referenced to make this a tracked dependency (see above).
    $: rollup = summarise(rev, node);

    const plural = (n: number, word: string): string =>
        `${n} ${word}${n === 1 ? "" : "s"}`;

    // A branch reads as the sum of its leaves: total leaf topics + concepts
    // beneath it. Authoring has no per-concept study state, so these are the
    // authored counts, not a mastery meter.
    function summarise(_rev: number, n: Node): { topics: number; concepts: number } {
        return { topics: countTopics(n), concepts: countConcepts(n) };
    }
    function countTopics(n: Node): number {
        return n.children.length === 0
            ? 1
            : n.children.reduce((sum, child) => sum + countTopics(child), 0);
    }
    function countConcepts(n: Node): number {
        return (
            n.concepts.length +
            n.children.reduce((sum, child) => sum + countConcepts(child), 0)
        );
    }

    // The edited node pulses first, then each ancestor in turn, root last.
    function selfChain(): string[] {
        return [node.id, ...[...ancestors].reverse()];
    }

    function onTitleInput(): void {
        ctx.change();
    }
    function onKeydown(event: KeyboardEvent): void {
        if (event.key === "Enter") {
            event.preventDefault();
            (event.currentTarget as HTMLInputElement).blur();
        }
    }

    async function addChild(): Promise<void> {
        const child = newNode("");
        node.children = [...node.children, child];
        ctx.change();
        ctx.pulse([child.id, ...selfChain()]);
        await tick();
        document.getElementById(`sr-node-${child.id}`)?.focus();
    }

    function removeChild(id: string): void {
        node.children = node.children.filter((c) => c.id !== id);
        ctx.change();
        ctx.pulse(selfChain());
    }

    // Focusing a leaf's name opens it: the concepts list follows the open leaf,
    // and the same field renames it. Branch names only rename.
    function onNameFocus(): void {
        if (leaf) {
            ctx.select(node.id);
        }
    }
</script>

<li class="node">
    <div
        class="row"
        class:root={isRoot}
        class:branch={!isRoot && hasChildren}
        class:leaf
        class:selected={leaf && selected}
        class:active={isSelectedParent}
    >
        {#if $pulseState && pulseIndex >= 0}
            {#key $pulseState.seq}
                <span
                    class="pulse"
                    style="--pulse-delay: {pulseIndex * STAGGER_MS}ms"
                    aria-hidden="true"
                ></span>
            {/key}
        {/if}

        {#if isRoot || hasChildren}
            <span class="caret" aria-hidden="true">▾</span>
        {/if}

        {#if isRoot}
            <span class="name name-text">{node.title.trim() || "Untitled deck"}</span>
        {:else}
            <input
                id={`sr-node-${node.id}`}
                class="name name-input"
                bind:value={node.title}
                on:input={onTitleInput}
                on:keydown={onKeydown}
                on:focus={onNameFocus}
                placeholder="Name"
                aria-label={leaf ? "Topic name" : "Group name"}
            />
        {/if}

        {#if isRoot || hasChildren}
            {#if hasChildren}
                <span class="rollup">
                    {plural(rollup.topics, "topic")}
                    <span class="dot" aria-hidden="true">·</span>
                    {plural(rollup.concepts, "concept")}
                </span>
            {/if}
        {:else}
            <span class="leaf-meta">{plural(node.concepts.length, "concept")}</span>
        {/if}

        {#if leaf}
            <div class="controls">
                <button class="mini" on:click={addChild} aria-label="Add subtopic">
                    +
                </button>
                {#if onRemove}
                    <button class="mini" on:click={onRemove} aria-label="Delete topic">
                        ×
                    </button>
                {/if}
            </div>
        {:else if onRemove}
            <button class="del" on:click={onRemove} aria-label="Delete topic">×</button>
        {/if}
    </div>

    {#if isRoot || hasChildren}
        <ul class="children" class:root-children={isRoot}>
            {#each node.children as child (child.id)}
                <svelte:self
                    node={child}
                    ancestors={childAncestors}
                    depth={depth + 1}
                    {rev}
                    onRemove={() => removeChild(child.id)}
                />
            {/each}
            <li class="affordance-item">
                <button class="affordance" class:topic={isRoot} on:click={addChild}>
                    <span class="plus" aria-hidden="true">+</span>
                    {isRoot ? "Add topic" : "Add subtopic"}
                </button>
            </li>
        </ul>
    {/if}
</li>

<style lang="scss">
    .node {
        list-style: none;
    }

    .row {
        position: relative;
        display: flex;
        align-items: center;
        gap: 12px;
    }

    // Deck root: a white node with the strong control edge.
    .row.root {
        background: var(--sr-white);
        border: 1px solid var(--sr-line-strong);
        border-radius: 12px;
        padding: 12px 16px;
    }
    // Group branch: a nested surface, tinting coral only while it is the direct
    // parent of the open leaf, so at most one branch ever reads as active.
    // Editing a branch name shows its own coral underline, not a row highlight,
    // so focus never leaves a second branch stuck coral.
    .row.branch {
        background: var(--sr-panel-2);
        border: 1.5px solid transparent;
        border-radius: 11px;
        padding: 11px 14px;
    }
    .row.branch.active {
        background: var(--sr-white);
        border-color: var(--sr-signal);
    }
    // Leaf topic: the near-white builder row.
    .row.leaf {
        background: var(--sr-tile);
        border: 1.5px solid var(--sr-line);
        border-radius: 10px;
        padding: 9px 12px;
    }
    .row.leaf.selected {
        background: var(--sr-white);
        border-color: var(--sr-signal);
    }

    .caret {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--sr-ink-2);
    }
    .row.root .caret {
        width: 22px;
        height: 22px;
        border-radius: 7px;
        background: var(--sr-inset);
        font-size: 11px;
    }
    .row.branch .caret {
        width: 20px;
        height: 20px;
        border-radius: 6px;
        background: var(--sr-track);
        font-size: 10px;
    }
    .row.branch.active .caret {
        background: var(--sr-signal-weak);
        color: var(--sr-signal);
    }
    :global(.night-mode) .row.branch.active .caret {
        color: var(--sr-signal-ink);
    }

    .name {
        flex: 1;
        min-width: 0;
        color: var(--sr-ink);
    }
    .name-text {
        font-weight: 700;
        font-size: 14px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    // Inline rename: a quiet field that shows a coral dashed underline on focus,
    // never a box.
    .name-input {
        border: none;
        border-bottom: 1px dashed transparent;
        background: none;
        padding: 2px 1px;
        font-family: var(--sr-sans);
        color: var(--sr-ink);
        caret-color: var(--sr-signal);
    }
    .row.branch .name-input {
        font-weight: 600;
        font-size: 13px;
    }
    .row.leaf .name-input {
        font-weight: 500;
        font-size: 12.5px;
    }
    .name-input::placeholder {
        color: var(--sr-faint);
    }
    .name-input:focus {
        outline: none;
        border-bottom-color: var(--sr-signal);
    }

    .rollup {
        flex-shrink: 0;
        margin-left: auto;
        font-family: var(--sr-mono);
        font-size: 9.5px;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
        white-space: nowrap;
    }
    .row.root .rollup {
        font-size: 10px;
    }
    .rollup .dot {
        margin: 0 0.2em;
    }
    .leaf-meta {
        flex-shrink: 0;
        margin-left: auto;
        font-family: var(--sr-mono);
        font-size: 10px;
        color: var(--sr-faint);
        font-variant-numeric: tabular-nums;
        white-space: nowrap;
    }

    // Per-leaf controls stay hidden until the row is hovered or focused.
    .controls {
        flex-shrink: 0;
        display: flex;
        gap: 4px;
        opacity: 0;
        transition: opacity 0.12s ease;
    }
    .row.leaf:hover .controls,
    .row.leaf:focus-within .controls,
    .row.leaf.selected .controls {
        opacity: 1;
    }
    .mini {
        width: 22px;
        height: 22px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-white);
        color: var(--sr-ink-3);
        border-radius: 7px;
        font-size: 0.9rem;
        line-height: 1;
        cursor: pointer;
    }
    .mini:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }

    // Branch / root delete: quiet until the row is hovered or focused.
    .del {
        flex-shrink: 0;
        width: 22px;
        height: 22px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-white);
        border-radius: 7px;
        color: var(--sr-ink-3);
        font-size: 1rem;
        line-height: 1;
        cursor: pointer;
        opacity: 0;
        transition: opacity 0.12s ease;
    }
    .row:hover .del,
    .row:focus-within .del {
        opacity: 1;
    }
    .del:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }

    // Indent guide: one rail per level, read from the indent alone.
    .children {
        list-style: none;
        margin: 6px 0 0;
        padding: 0 0 0 14px;
        margin-left: 14px;
        border-left: 2px solid var(--sr-line-strong);
        display: flex;
        flex-direction: column;
        gap: 5px;
    }
    .children.root-children {
        margin-left: 16px;
        padding-left: 16px;
        gap: 6px;
    }
    .affordance-item {
        list-style: none;
    }

    // Explicit create affordances: dashed, quiet, sized to their label.
    .affordance {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        width: fit-content;
        border: 1.5px dashed var(--sr-line-dashed);
        border-radius: 10px;
        background: none;
        padding: 7px 12px;
        font-family: var(--sr-sans);
        font-size: 12px;
        font-weight: 500;
        color: var(--sr-ink-3);
        cursor: pointer;
    }
    .affordance.topic {
        border-radius: 11px;
        padding: 9px 14px;
        font-size: 12.5px;
        font-weight: 600;
    }
    .affordance:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }
    .affordance .plus {
        font-size: 1em;
        line-height: 1;
    }

    // Rollup pulse: a coral tick on each row's left edge, re-keyed on every edit
    // and staggered by ancestor depth so it climbs from the edited node up to the
    // deck.
    .pulse {
        position: absolute;
        left: 0;
        top: 5px;
        bottom: 5px;
        width: 2px;
        border-radius: 2px;
        background: var(--sr-signal);
        pointer-events: none;
        animation: sr-pulse 600ms ease-out both;
        animation-delay: var(--pulse-delay, 0ms);
    }
    @keyframes sr-pulse {
        0% {
            opacity: 0;
        }
        25% {
            opacity: 0.9;
        }
        100% {
            opacity: 0;
        }
    }
    @media (prefers-reduced-motion: reduce) {
        .pulse {
            display: none;
        }
        .controls,
        .del {
            transition: none;
        }
    }

    // Phone: reclaim the horizontal room the deep indent + row padding take, so
    // topic/group names get enough width to read instead of clipping.
    @media (max-width: 34rem) {
        .row {
            gap: 8px;
        }
        .row.root {
            padding: 11px 12px;
        }
        .row.branch {
            padding: 10px 10px;
        }
        .row.leaf {
            padding: 9px 10px;
        }
        .children,
        .children.root-children {
            margin-left: 8px;
            padding-left: 10px;
        }
    }
</style>
