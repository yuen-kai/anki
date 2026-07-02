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

    const ctx = getTreeContext();
    const { selectedId, pulseState } = ctx;

    const STAGGER_MS = 70;

    $: leaf = node.children.length === 0;
    $: selected = $selectedId === node.id;
    $: childAncestors = [...ancestors, node.id];
    $: pulseIndex = $pulseState ? $pulseState.ids.indexOf(node.id) : -1;

    // The edited node pulses first, then each ancestor in turn, root last.
    function selfChain(): string[] {
        return [node.id, ...[...ancestors].reverse()];
    }

    function onTitleInput(): void {
        ctx.change();
    }

    function commitTitle(): void {
        ctx.change();
        ctx.pulse(selfChain());
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
</script>

<li class="node" class:child={depth > 0}>
    {#if $pulseState && pulseIndex >= 0 && depth > 0}
        {#key $pulseState.seq}
            <span
                class="climb"
                style="--pulse-delay: {pulseIndex * STAGGER_MS}ms"
                aria-hidden="true"
            ></span>
        {/key}
    {/if}
    <div class="row" class:selected={selected && leaf}>
        <label class="sr-only" for={`sr-node-${node.id}`}>
            {depth === 0 ? "Deck name" : "Topic name"}
        </label>
        <input
            id={`sr-node-${node.id}`}
            class="title-input"
            class:root={depth === 0}
            bind:value={node.title}
            on:input={onTitleInput}
            on:change={commitTitle}
            on:keydown={onKeydown}
            placeholder={depth === 0 ? "Deck name" : "Topic"}
        />

        <div class="controls">
            {#if leaf}
                <button
                    class="concepts"
                    class:active={selected}
                    on:click={() => ctx.select(node.id)}
                >
                    <span>Concepts</span>
                    {#if node.concepts.length}
                        <span class="count">{node.concepts.length}</span>
                    {/if}
                </button>
            {/if}
            <button class="ctrl" on:click={addChild} aria-label="Add sub-topic">
                +
            </button>
            {#if onRemove}
                <button class="ctrl del" on:click={onRemove} aria-label="Delete topic">
                    ×
                </button>
            {/if}
        </div>
    </div>

    {#if node.children.length}
        <ul class="children">
            {#each node.children as child (child.id)}
                <svelte:self
                    node={child}
                    ancestors={childAncestors}
                    depth={depth + 1}
                    onRemove={() => removeChild(child.id)}
                />
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
        display: flex;
        align-items: center;
        gap: 0.4rem;
        min-height: 2.1rem;
        padding: 0 0.4rem;
        border-radius: 6px;
    }
    .row.selected {
        background: var(--sr-signal-weak);
        box-shadow: inset 0 0 0 1px var(--sr-signal-line);
    }

    // An inline editable label on the schematic: an underline, never a box.
    .title-input {
        flex: 1 1 auto;
        min-width: 4rem;
        border: none;
        border-bottom: 1px solid transparent;
        background: none;
        padding: 0.25rem 0.1rem;
        font: inherit;
        color: var(--sr-ink);
    }
    .title-input::placeholder {
        color: var(--sr-ink-3);
    }
    .title-input:hover {
        border-bottom-color: var(--sr-line-2);
    }
    .title-input:focus {
        border-bottom-color: var(--sr-signal);
        outline: none;
    }
    .title-input.root {
        font-weight: 700;
        font-size: 1.15rem;
        letter-spacing: -0.01em;
    }

    .controls {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        flex-shrink: 0;
    }
    // Add/delete stay quiet until the row is hovered or its field is focused.
    .ctrl {
        width: 1.6rem;
        height: 1.6rem;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel);
        border-radius: 5px;
        color: var(--sr-ink-3);
        font-size: 1.05rem;
        line-height: 1;
        cursor: pointer;
        opacity: 0;
        transition:
            opacity 0.12s ease,
            color 0.12s ease,
            border-color 0.12s ease;
    }
    .row:hover .ctrl,
    .row:focus-within .ctrl {
        opacity: 1;
    }
    .ctrl:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }

    // Discoverable affordance: a marigold-outlined chip, always visible.
    .concepts {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        border: 1px solid var(--sr-signal-line);
        background: var(--sr-panel);
        border-radius: 999px;
        padding: 0.18rem 0.6rem;
        font: inherit;
        font-size: 0.78rem;
        font-weight: 560;
        color: var(--sr-ink-2);
        cursor: pointer;
        white-space: nowrap;
    }
    .concepts:hover,
    .concepts.active {
        background: var(--sr-signal-weak);
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .concepts:hover,
    :global(.night-mode) .concepts.active {
        color: var(--sr-signal);
    }
    .count {
        font-family: var(--sr-mono);
        font-size: 0.68rem;
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink-2);
        background: var(--sr-panel-2);
        border-radius: 999px;
        padding: 0.02rem 0.36rem;
    }

    // The pulse: a marigold glow shaped like the incoming connector, re-keyed on
    // each edit and staggered by ancestor depth so it climbs UP the spine.
    .climb {
        position: absolute;
        left: 0;
        top: 0;
        width: 1.1rem;
        height: 1.05rem;
        border-left: 1.5px solid var(--sr-signal);
        border-bottom: 1.5px solid var(--sr-signal);
        pointer-events: none;
        animation: sr-climb 600ms ease-out both;
        animation-delay: var(--pulse-delay, 0ms);
    }
    @keyframes sr-climb {
        0% {
            opacity: 0;
        }
        20% {
            opacity: 1;
        }
        100% {
            opacity: 0;
        }
    }
    @media (prefers-reduced-motion: reduce) {
        .climb {
            display: none;
        }
    }

    // Blueprint spine: one vertical line per level with right-angle elbows to
    // each child; the last child's spine stops at its elbow to draw a clean L.
    .children {
        list-style: none;
        margin: 0;
        padding: 0;
    }
    // `.child` (depth > 0) draws its own incoming connector; scoping it to the
    // node's own class keeps svelte-check happy across the recursive component.
    .node.child {
        position: relative;
        padding-left: 1.5rem;
    }
    .node.child::before {
        content: "";
        position: absolute;
        left: 0;
        top: 1.05rem;
        width: 1.1rem;
        border-top: 1.5px solid var(--sr-line-2);
        pointer-events: none;
    }
    .node.child::after {
        content: "";
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        border-left: 1.5px solid var(--sr-line-2);
        pointer-events: none;
    }
    .node.child:last-child::after {
        bottom: auto;
        height: 1.05rem;
    }

    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
    }
</style>
