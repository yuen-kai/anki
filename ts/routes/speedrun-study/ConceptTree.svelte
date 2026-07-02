<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { MASTERY_STAGES } from "../speedrun-dashboard/lib";
    import ConceptTreeNode from "./ConceptTreeNode.svelte";
    import type { ConceptTreeNode as TreeNode } from "./lib";

    export let tree: TreeNode | null;
    export let error: string | null = null;
</script>

{#if error}
    <div class="state">
        <p class="headline">Couldn't load the concept tree</p>
        <p class="detail">{error}</p>
    </div>
{:else if !tree}
    <div class="state">
        <p class="headline">No topics yet</p>
        <p class="detail">
            Add a hierarchy to this deck, or study it to start tracking its topics.
        </p>
    </div>
{:else}
    <div class="tree-scroll">
        <ul class="sr-tree" aria-label="Concept tree">
            <ConceptTreeNode node={tree} />
        </ul>
    </div>

    <ul class="legend">
        {#each MASTERY_STAGES as stage, i}
            <li>
                <span class="dot s{i}"></span>
                {stage.label}
            </li>
        {/each}
        <li>
            <span class="dot cold"></span>
            Not started
        </li>
    </ul>
{/if}

<style lang="scss">
    // Centre the whole tree in the page column; it scrolls horizontally only if a
    // wide tree overflows on a small window (it unwraps to a list under 40rem).
    .tree-scroll {
        overflow-x: auto;
        padding: 0.5rem 0 0.25rem;
    }
    .sr-tree {
        display: flex;
        justify-content: center;
        min-width: min-content;
        margin: 0 auto;
        padding: 0;
        list-style: none;
    }

    .legend {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        gap: 0.4rem 1.1rem;
        margin: 1.4rem 0 0;
        padding: 0;
        list-style: none;
    }
    .legend li {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .dot {
        width: 9px;
        height: 6px;
        border-radius: 1px;
        background: var(--sr-line-2);
    }
    .dot.s1 {
        background: var(--sr-ink-3);
    }
    .dot.s2 {
        background: var(--sr-ink-2);
    }
    .dot.s3 {
        background: var(--sr-signal);
    }
    .dot.cold {
        background: transparent;
        border: 1px dashed var(--sr-line-2);
    }

    .state {
        text-align: center;
        padding: 1.5rem 0;
    }
    .state .headline {
        margin: 0 0 0.4rem;
        font-size: 1rem;
        font-weight: 600;
    }
    .state .detail {
        margin: 0 auto;
        max-width: 42ch;
        font-size: 0.9rem;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }

    @media (max-width: 48rem) {
        .sr-tree {
            justify-content: flex-start;
            min-width: 0;
        }
        .legend {
            justify-content: flex-start;
        }
    }
</style>
