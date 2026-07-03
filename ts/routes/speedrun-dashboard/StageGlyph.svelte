<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { STAGE_COUNT, STAGE_OFF_COLOR, stageColor } from "./lib";

    // 0-based current stage (0..STAGE_COUNT-1); null = not started, all inactive.
    export let stage: number | null = null;
    // Segment scale, matching the canvas: sm = the inline stage-tag glyph
    // (7x3), md = the tree-leaf / card glyph (8x4), lg = the topic-learned /
    // stepper glyph (9x4). Orientation stacks the segments; horizontal is the
    // default used on cards, tree leaves and the level-up stepper.
    export let size: "sm" | "md" | "lg" = "md";
    export let orientation: "horizontal" | "vertical" = "horizontal";
    // Fill the parent's width (equal flex segments) instead of fixed-width chips.
    export let stretch = false;
    // When set, the glyph is announced with this label; otherwise it's decorative.
    export let label: string | null = null;

    // Monochrome, matching the canvas: every lit segment (index <= the current
    // stage) takes the CURRENT stage's single colour, the rest the inactive
    // colour. A Guided glyph is three amber segments, not grey/blue/amber.
    $: lit = stageColor(stage);
    $: colors = Array.from({ length: STAGE_COUNT }, (_, i) =>
        stage !== null && i <= stage ? lit : STAGE_OFF_COLOR,
    );
</script>

<span
    class="glyph {size} {orientation}"
    class:stretch
    role={label ? "img" : undefined}
    aria-label={label ?? undefined}
    aria-hidden={label ? undefined : true}
>
    {#each colors as color, i (i)}
        <span class="seg" style="--seg: {color}"></span>
    {/each}
</span>

<style lang="scss">
    .glyph {
        display: inline-flex;
        gap: var(--seg-gap);
        vertical-align: middle;
        line-height: 0;
    }
    .glyph.vertical {
        flex-direction: column;
    }
    .glyph.stretch {
        display: flex;
        width: 100%;
    }

    // Exact canvas segment dimensions; every glyph uses a 2px gap + 2px radius.
    .sm {
        --seg-w: 7px;
        --seg-h: 3px;
        --seg-gap: 2px;
        --seg-radius: 2px;
    }
    .md {
        --seg-w: 8px;
        --seg-h: 4px;
        --seg-gap: 2px;
        --seg-radius: 2px;
    }
    .lg {
        --seg-w: 9px;
        --seg-h: 4px;
        --seg-gap: 2px;
        --seg-radius: 2px;
    }

    .seg {
        flex: 0 0 auto;
        width: var(--seg-w);
        height: var(--seg-h);
        border-radius: var(--seg-radius);
        background: var(--seg);
    }
    .glyph.stretch .seg {
        flex: 1 1 0;
        width: auto;
    }
    .glyph.stretch.vertical .seg {
        height: auto;
    }
</style>
