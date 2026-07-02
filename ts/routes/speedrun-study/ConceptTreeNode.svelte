<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { STAGE_COUNT } from "../speedrun-dashboard/lib";
    import type { ConceptTreeNode } from "./lib";

    export let node: ConceptTreeNode;

    // Filled pips: none for a "not started" leaf, else stage + 1 (learning fills
    // one, mastering all four), matching the dashboard topic meter.
    $: filled = node.stage === null ? 0 : node.stage + 1;
    $: mastered = node.stage === STAGE_COUNT - 1;
</script>

<li class:leaf={node.isLeaf} class:branch={!node.isLeaf}>
    <div
        class="card"
        class:done={mastered}
        class:cold={node.isLeaf && node.stage === null}
    >
        <span class="title">{node.title}</span>

        {#if node.isLeaf}
            <span
                class="meter"
                role="img"
                aria-label={node.stage === null
                    ? "Not started"
                    : `${node.stageLabel}, stage ${node.stage + 1} of ${STAGE_COUNT}`}
            >
                {#each Array(STAGE_COUNT) as _pip, i}
                    <i
                        class:on={i < filled}
                        class:top={mastered && i === filled - 1}
                    ></i>
                {/each}
            </span>
            <span class="stage" class:done={mastered} class:cold={node.stage === null}>
                {node.stageLabel}
            </span>
        {:else}
            <span
                class="agg"
                aria-label="{Math.round(
                    node.fraction * 100,
                )}% across {node.leafCount} topics"
            >
                <span class="agg-bar">
                    <span class="agg-fill" style="width:{node.fraction * 100}%"></span>
                </span>
                <span class="agg-count">{node.leafCount}</span>
            </span>
        {/if}
    </div>

    {#if node.children.length}
        <ul>
            {#each node.children as child (child.id)}
                <svelte:self node={child} />
            {/each}
        </ul>
    {/if}
</li>

<style lang="scss">
    // A centred org-tree: each level is a flex row centred under its parent, with
    // drawn connectors. On narrow screens it unwraps into a left-indented list.
    ul {
        display: flex;
        justify-content: center;
        position: relative;
        margin: 0;
        padding: 20px 0 0;
        list-style: none;
    }
    // The vertical drop from a parent card down to its children's row.
    ul::before {
        content: "";
        position: absolute;
        top: 0;
        left: 50%;
        width: 0;
        height: 20px;
        border-left: 1px solid var(--sr-line-2);
    }

    li {
        position: relative;
        padding: 20px 5px 0;
        text-align: center;
    }
    // Horizontal connectors joining siblings to the drop line.
    li::before,
    li::after {
        content: "";
        position: absolute;
        top: 0;
        right: 50%;
        width: 50%;
        height: 20px;
        border-top: 1px solid var(--sr-line-2);
    }
    li::after {
        right: auto;
        left: 50%;
        border-left: 1px solid var(--sr-line-2);
    }
    // A lone child needs no elbow, just the straight drop above it.
    li:only-child::before,
    li:only-child::after {
        display: none;
    }
    li:only-child {
        padding-top: 20px;
    }
    // The root card (only child of the outer list) sits flush.
    :global(.sr-tree) > li:only-child {
        padding-top: 0;
    }
    // Trim the outer half-connectors so the elbows read as brackets.
    li:first-child::before,
    li:last-child::after {
        border: 0 none;
    }
    li:last-child::before {
        border-right: 1px solid var(--sr-line-2);
    }

    .card {
        display: inline-flex;
        flex-direction: column;
        align-items: center;
        gap: 0.35rem;
        min-width: 4.5rem;
        max-width: 7.25rem;
        padding: 0.5rem 0.5rem;
        border: 1px solid var(--sr-line);
        border-radius: 7px;
        background: var(--sr-panel);
    }
    // Branches read as structure; leaves as the scored unit.
    .branch > .card {
        background: var(--sr-panel-2);
        border-color: var(--sr-line-2);
    }
    .card.done {
        border-color: var(--sr-signal-line);
    }
    .card.cold {
        border-style: dashed;
    }

    .title {
        font-size: 0.8rem;
        font-weight: 560;
        line-height: 1.25;
        overflow-wrap: anywhere;
    }

    .meter {
        display: inline-flex;
        gap: 3px;
    }
    .meter i {
        width: 12px;
        height: 6px;
        border-radius: 1px;
        background: var(--sr-line-2);
    }
    .meter i.on {
        background: var(--sr-ink-2);
    }
    .meter i.top {
        background: var(--sr-signal);
    }

    .stage {
        font-family: var(--sr-mono);
        font-size: 9px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .stage.done {
        color: var(--sr-signal);
    }
    .stage.cold {
        color: var(--sr-ink-3);
    }

    .agg {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        width: 100%;
    }
    .agg-bar {
        flex: 1;
        height: 5px;
        border-radius: 3px;
        background: var(--sr-line);
        overflow: hidden;
    }
    .agg-fill {
        display: block;
        height: 100%;
        background: var(--sr-signal);
    }
    .agg-count {
        font-family: var(--sr-mono);
        font-size: 10px;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }

    @media (max-width: 48rem) {
        ul {
            flex-direction: column;
            align-items: stretch;
            padding: 0.35rem 0 0 0.9rem;
            margin-left: 0.5rem;
            border-left: 1px solid var(--sr-line);
        }
        ul::before {
            display: none;
        }
        li,
        li:only-child {
            padding: 0.4rem 0 0;
            text-align: left;
        }
        li::before,
        li::after {
            display: none;
        }
        .card {
            flex-direction: row;
            justify-content: space-between;
            align-items: center;
            width: 100%;
            max-width: none;
        }
        .agg {
            width: auto;
            min-width: 5rem;
        }
    }
</style>
