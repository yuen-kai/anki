<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

One level of the new-topic spine: the on-path node dead-centre, its off-path
siblings flanking it (quiet), and the coral marching-dash connector that links
it down from the level above. NewTopicIntro stacks one of these per path level.
-->
<script lang="ts">
    import type { Node } from "./lib";

    export let node: Node;
    export let kind: "root" | "middle" | "target";
    // The parent's children, so we can flank the path node with its siblings.
    // Empty for the root, which has no siblings to show.
    export let siblings: Node[] = [];
    // Draw the connector above this level (every level except the root).
    export let connector = false;

    $: index = siblings.findIndex((sibling) => sibling.id === node.id);
    $: left = index >= 0 ? siblings.slice(0, index) : [];
    $: right = index >= 0 ? siblings.slice(index + 1) : [];
    $: title = node.title.trim() || "Untitled";
</script>

<div class="level">
    {#if connector}
        <span class="link" aria-hidden="true"></span>
    {/if}

    <div class="row">
        <div class="flank end">
            {#each left as sibling (sibling.id)}
                <span class="tile quiet" class:leaf={sibling.children.length === 0}>
                    {sibling.title.trim() || "Untitled"}
                </span>
            {/each}
        </div>

        <div class="mid">
            {#if kind === "target"}
                <span class="tile target">
                    <span class="tt">{title}</span>
                    <span class="pill">New</span>
                </span>
            {:else}
                <span class="tile {kind}">{title}</span>
            {/if}
        </div>

        <div class="flank start">
            {#each right as sibling (sibling.id)}
                <span class="tile quiet" class:leaf={sibling.children.length === 0}>
                    {sibling.title.trim() || "Untitled"}
                </span>
            {/each}
        </div>
    </div>
</div>

<style lang="scss">
    .level {
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 100%;
    }

    // Coral marching dashes flowing down the path to the new leaf.
    .link {
        width: 2px;
        height: 28px;
        background-image: repeating-linear-gradient(
            to bottom,
            var(--sr-signal) 0 6px,
            transparent 6px 12px
        );
        background-size: 2px 12px;
        animation: sr-march 1.1s linear infinite;
    }
    @keyframes sr-march {
        to {
            background-position-y: 12px;
        }
    }

    // 1fr auto 1fr keeps the path node centred no matter the sibling counts.
    .row {
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        align-items: center;
        column-gap: 12px;
        width: 100%;
    }
    .flank {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 8px;
        min-width: 0;
    }
    .flank.end {
        justify-content: flex-end;
    }
    .flank.start {
        justify-content: flex-start;
    }
    .mid {
        display: flex;
        justify-content: center;
    }

    .tile {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        box-sizing: border-box;
        white-space: nowrap;
        font-family: var(--sr-sans);
    }

    .tile.root {
        background: var(--sr-white);
        color: var(--sr-ink);
        border: 1.5px solid var(--sr-signal);
        border-radius: var(--sr-radius-tile);
        padding: 9px 18px;
        font-size: 14px;
        font-weight: 700;
        box-shadow: 0 6px 18px color-mix(in srgb, var(--sr-signal) 18%, transparent);
    }

    .tile.middle {
        background: var(--sr-white);
        color: var(--sr-ink);
        border: 1.5px solid var(--sr-signal);
        border-radius: var(--sr-radius-control);
        padding: 8px 16px;
        font-size: 12.5px;
        font-weight: 700;
        box-shadow: 0 4px 14px color-mix(in srgb, var(--sr-signal) 25%, transparent);
    }

    .tile.target {
        gap: 8px;
        background: var(--sr-white);
        border: 2px solid var(--sr-signal);
        border-radius: var(--sr-radius-tile);
        padding: 9px 10px;
        box-shadow: 0 0 24px color-mix(in srgb, var(--sr-signal) 50%, transparent);
    }
    .tile.target .tt {
        font-size: 12px;
        font-weight: 700;
        color: var(--sr-ink);
    }
    .pill {
        font-family: var(--sr-mono);
        font-size: 8px;
        font-weight: 600;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        padding: 3px 6px;
        border-radius: 5px;
        // Text rides on the coral fill, so it stays literally white in both
        // themes (--sr-white is a surface token that flips dark at night).
        background: var(--sr-signal);
        color: #fff;
    }

    // Off-path branches: quiet groups, quiet leaves.
    .tile.quiet {
        background: var(--sr-panel-2);
        color: var(--sr-ink-3);
        border-radius: var(--sr-radius-control);
        padding: 8px 14px;
        font-size: 12.5px;
        font-weight: 600;
        opacity: 0.75;
    }
    .tile.quiet.leaf {
        background: var(--sr-panel);
        color: var(--sr-faint);
        border: 1px solid var(--sr-line);
        width: 124px;
        padding: 9px 8px;
        font-size: 11.5px;
        white-space: normal;
        text-align: center;
        opacity: 1;
    }

    @media (prefers-reduced-motion: reduce) {
        .link {
            animation: none;
        }
    }
</style>
