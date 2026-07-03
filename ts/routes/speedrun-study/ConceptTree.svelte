<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { MASTERY_STAGES, stageColor } from "../speedrun-dashboard/lib";
    import ConceptTreeNode from "./ConceptTreeNode.svelte";
    import {
        type ConceptTreeNode as TreeNode,
        flattenLeaves,
        rollupColor,
    } from "./lib";

    export let tree: TreeNode | null;
    export let error: string | null = null;

    $: groups = tree ? tree.children : [];
    const pct = (fraction: number): number => Math.round(fraction * 100);
</script>

<div class="head">
    <h2 class="eyebrow">Concept tree</h2>
    {#if tree && !error}
        <ul class="legend" aria-hidden="true">
            {#each MASTERY_STAGES as stage, i (stage.state)}
                <li>
                    <span class="sw" style="background:{stageColor(i)}"></span>
                    {stage.label}
                </li>
            {/each}
        </ul>
    {/if}
</div>

{#if error}
    <p class="state">Couldn't load the concept tree. {error}</p>
{:else if tree}
    <!-- Desktop: deck → group columns → topic rows. -->
    <div class="desktop">
        <div class="root">
            <span class="root-name">{tree.title}</span>
            <span class="root-bar">
                <span class="root-fill" style="width:{pct(tree.fraction)}%"></span>
            </span>
            <span class="root-pct">{pct(tree.fraction)}%</span>
        </div>

        {#if groups.length}
            <div class="drop-main"></div>
            <div class="groups">
                {#each groups as group (group.id)}
                    <div class="group">
                        <div class="group-head">
                            <div class="group-top">
                                <span class="group-name">{group.title}</span>
                                <span class="group-pct">{pct(group.fraction)}%</span>
                            </div>
                            <span class="group-bar">
                                <span
                                    class="group-fill"
                                    style="width:{pct(
                                        group.fraction,
                                    )}%;background:{rollupColor(group.fraction)}"
                                ></span>
                            </span>
                        </div>
                        {#each flattenLeaves(group) as leaf, i (leaf.id)}
                            <div class="conn" class:first={i === 0}></div>
                            <ConceptTreeNode node={leaf} />
                        {/each}
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Phone: one row per group, no leaves. -->
    <div class="phone">
        {#each groups as group (group.id)}
            <div class="grow">
                <span class="grow-name">{group.title}</span>
                <span class="grow-bar">
                    <span
                        class="grow-fill"
                        style="width:{pct(group.fraction)}%;background:{rollupColor(
                            group.fraction,
                        )}"
                    ></span>
                </span>
                <span class="grow-pct">{pct(group.fraction)}%</span>
            </div>
        {/each}
    </div>
{/if}

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    // Column geometry the connector bus is drawn against: the bus spans from the
    // first column's centre to the last's, so it insets half a column each side.
    $col: 320px;
    $col-half: 160px;

    .head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        margin-bottom: 20px;
    }
    .eyebrow {
        margin: 0;
        @include syn.eyebrow;
    }
    .legend {
        display: flex;
        flex-wrap: wrap;
        gap: 14px;
        margin: 0;
        padding: 0;
        list-style: none;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 9.5px;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .legend li {
        display: inline-flex;
        align-items: center;
    }
    .sw {
        width: 8px;
        height: 8px;
        margin-right: 4px;
        border-radius: 3px;
    }

    .state {
        margin: 0;
        font-size: 0.9rem;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }

    // ── Desktop tree ─────────────────────────────────────────────────────────
    .desktop {
        display: flex;
        flex-direction: column;
        align-items: center;
        overflow-x: auto;
    }

    .root {
        @include syn.tree-root;
    }
    .root-name {
        font-weight: 700;
        font-size: 15px;
    }
    .root-bar {
        display: inline-block;
        width: 130px;
        height: 7px;
        border-radius: 5px;
        background: var(--sr-track);
        overflow: hidden;
    }
    .root-fill {
        display: block;
        height: 100%;
        background: var(--sr-signal);
    }
    .root-pct {
        font-family: var(--sr-mono);
        font-weight: 600;
        font-size: 12px;
        color: var(--sr-signal);
    }

    // Two 22px runs sit between the deck node and the group headers: the drop from
    // the deck, then the horizontal bus with a drop into each column.
    .drop-main {
        width: 2px;
        height: 22px;
        background: var(--sr-line-connector);
    }
    .groups {
        position: relative;
        display: flex;
        gap: 40px;
    }
    .groups::before {
        content: "";
        position: absolute;
        top: 0;
        left: $col-half;
        right: $col-half;
        height: 2px;
        background: var(--sr-line-connector);
    }

    .group {
        position: relative;
        width: $col;
        padding-top: 22px;
        display: flex;
        flex-direction: column;
        align-items: center;
    }
    .group::before {
        content: "";
        position: absolute;
        top: 0;
        left: calc(50% - 1px);
        width: 2px;
        height: 22px;
        background: var(--sr-line-connector);
    }

    .group-head {
        width: 100%;
        box-sizing: border-box;
        @include syn.tree-group;
    }
    .group-top {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
    }
    .group-name {
        font-weight: 700;
        font-size: 13.5px;
    }
    .group-pct {
        font-family: var(--sr-mono);
        font-weight: 600;
        font-size: 11px;
        color: var(--sr-ink-3);
    }
    .group-bar {
        display: block;
        height: 5px;
        margin-top: 8px;
        border-radius: 3px;
        background: var(--sr-track);
        overflow: hidden;
    }
    .group-fill {
        display: block;
        height: 100%;
        border-radius: 3px;
    }

    .conn {
        width: 2px;
        height: 12px;
        background: var(--sr-line-connector);
    }
    .conn.first {
        height: 16px;
    }

    // ── Phone tree ───────────────────────────────────────────────────────────
    .phone {
        display: none;
        flex-direction: column;
        gap: 8px;
    }
    .grow {
        display: flex;
        align-items: center;
        gap: 9px;
    }
    .grow-name {
        flex: 1;
        min-width: 0;
        font-size: 12.5px;
        font-weight: 600;
    }
    .grow-bar {
        display: inline-block;
        width: 70px;
        height: 5px;
        border-radius: 3px;
        background: var(--sr-track);
        overflow: hidden;
    }
    .grow-fill {
        display: block;
        height: 100%;
    }
    .grow-pct {
        font-family: var(--sr-mono);
        font-weight: 600;
        font-size: 10px;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }

    @media (max-width: 40rem) {
        .desktop {
            display: none;
        }
        .phone {
            display: flex;
        }
    }
</style>
