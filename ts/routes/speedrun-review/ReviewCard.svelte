<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The session card shell shared by Learn / Practice / Guided / Solo. A stage-tag
header sits above the card; the card itself is a single Synapse surface with
`overflow:hidden` whose children are full-bleed steps (`.sc-sec`) divided by
seams, disclosed downward as the learner progresses. Steps set their own tone:
`--reveal` for a disclosed near-white section, `--grade` for the grading band.
-->
<div class="session">
    {#if $$slots.header}
        <div class="head">
            <slot name="header" />
        </div>
    {/if}
    <div class="card">
        <slot />
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .session {
        width: 100%;
        max-width: 47.5rem; // 760px, the canvas session-card measure
        display: flex;
        flex-direction: column;
        gap: 12px;
    }
    // Stage tag (+ optional count), aligned to the card's left edge.
    .head {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 0 2px;
    }
    .card {
        @include syn.card;
        overflow: hidden;

        // Full-bleed step, own padding so seams meet the card edges. Slotted by
        // the state components; styled here so every session card stays in step.
        :global(.sc-sec) {
            padding: 16px 18px;
        }
        // A disclosed step reads on the near-white reveal surface.
        :global(.sc-sec--reveal) {
            background: var(--sr-reveal);
        }
        // The grading step rides on its own cool fill at the card bottom.
        :global(.sc-sec--grade) {
            background: var(--sr-grading);
        }
    }

    @media (max-width: 34rem) {
        .card :global(.sc-sec) {
            padding: 14px 15px;
        }
    }
</style>
