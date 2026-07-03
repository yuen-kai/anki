<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The progressive-disclosure seam: a full-bleed dashed band ("▼ label") that
divides the sections of a session card, marking where the next step reveals
BELOW on the same card (never a new page). It is a divider only; the revealed
content is the sibling section that follows it. Used by Learn, Practice,
Guided and Solo between every disclosed step.
-->
<script lang="ts">
    // The words after the caret; states what opens below (e.g.
    // "details appear below · same screen").
    export let label = "same screen";
</script>

<div class="seam" role="separator" aria-label={label}>
    <span class="arrow" aria-hidden="true">▼</span>
    <span class="label">{label}</span>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .seam {
        @include syn.seam;
    }
    .arrow {
        color: var(--sr-signal);
        font-size: 10px;
        animation: sr-seam-nudge 520ms ease both;
    }
    @keyframes sr-seam-nudge {
        from {
            transform: translateY(-2px);
            opacity: 0;
        }
        60% {
            transform: translateY(1px);
        }
        to {
            transform: translateY(0);
            opacity: 1;
        }
    }
    .label {
        @include syn.seam-label;
    }
    @media (prefers-reduced-motion: reduce) {
        .arrow {
            animation: none;
        }
    }
</style>
