<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import {
        capitalize,
        displayNumber,
        displayRange,
        gaugeBounds,
        type ScoreEnvelope,
    } from "../speedrun-dashboard/lib";
    import { ringDash, ringFraction } from "./lib";

    export let name: string;
    export let envelope: ScoreEnvelope;
    // Whether the score is computed live; drives the abstain wording only.
    export let live: boolean;
    export let error: string | null = null;

    const R = 52;
    const SIZE = 128;

    // The ring only fills when there is a real number. An abstaining or errored
    // score shows a dashed empty ring and its give-up reason, never a fake fill.
    $: delivered = !error && !envelope.abstained;
    $: dash = ringDash(ringFraction(envelope), R);
    $: unit = envelope.format === "points" ? "" : "%";
    $: bounds = gaugeBounds(envelope.format);
    $: label = delivered
        ? `${name}: ${displayNumber(envelope.estimate, envelope.format)}${unit}, likely ${displayRange(
              envelope.rangeLow,
              envelope.rangeHigh,
              envelope.format,
          )}`
        : `${name}: no read`;
</script>

<div class="ring" class:off={!delivered}>
    <div class="dial" role="img" aria-label={label}>
        <svg viewBox="0 0 {SIZE} {SIZE}" aria-hidden="true">
            <circle
                class="track"
                cx={SIZE / 2}
                cy={SIZE / 2}
                r={R}
                stroke-dasharray={delivered ? "none" : "2 7"}
            />
            {#if delivered}
                <circle
                    class="value"
                    cx={SIZE / 2}
                    cy={SIZE / 2}
                    r={R}
                    stroke-dasharray={dash.circumference}
                    stroke-dashoffset={dash.offset}
                    transform="rotate(-90 {SIZE / 2} {SIZE / 2})"
                />
            {/if}
        </svg>
        <div class="readout">
            {#if delivered}
                <span class="num">
                    {displayNumber(envelope.estimate, envelope.format)}
                    <span class="unit">{unit}</span>
                </span>
                <span class="range">
                    {displayRange(
                        envelope.rangeLow,
                        envelope.rangeHigh,
                        envelope.format,
                    )}
                </span>
            {:else}
                <span class="num empty">&mdash;</span>
                <span class="range">
                    {envelope.format === "points" ? bounds.join("\u2013") : "no read"}
                </span>
            {/if}
        </div>
    </div>

    <p class="name">{name}</p>
    {#if delivered}
        <p class="note">likely range</p>
    {:else if error}
        <p class="note">Couldn't load this score.</p>
    {:else}
        <p class="note">
            {live ? "Not enough data yet." : "Not available yet."}
            {#if envelope.abstainReason}{capitalize(envelope.abstainReason)}{/if}
        </p>
    {/if}
</div>

<style lang="scss">
    .ring {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        min-width: 0;
        padding: 0 0.5rem;
    }

    .dial {
        position: relative;
        width: 128px;
        height: 128px;
    }
    svg {
        width: 100%;
        height: 100%;
        display: block;
    }
    .track {
        fill: none;
        stroke: var(--sr-line);
        stroke-width: 9;
    }
    .off .track {
        stroke: var(--sr-line-2);
        stroke-width: 6;
        stroke-linecap: round;
    }
    .value {
        fill: none;
        stroke: var(--sr-signal);
        stroke-width: 9;
        stroke-linecap: round;
        transition: stroke-dashoffset 0.5s ease;
    }

    .readout {
        position: absolute;
        inset: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.15rem;
    }
    .num {
        font-size: 2rem;
        font-weight: 720;
        line-height: 0.95;
        letter-spacing: -0.03em;
        font-variant-numeric: tabular-nums;
    }
    .num .unit {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--sr-ink-2);
        margin-left: 0.05em;
    }
    .num.empty {
        color: var(--sr-ink-3);
    }
    .range {
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.02em;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }

    .name {
        margin: 0.75rem 0 0;
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-ink);
    }
    .note {
        margin: 0.3rem 0 0;
        max-width: 22ch;
        font-size: 0.78rem;
        line-height: 1.4;
        color: var(--sr-ink-2);
    }
    .off .note {
        color: var(--sr-ink-3);
    }

    @media (prefers-reduced-motion: reduce) {
        .value {
            transition: none;
        }
    }
</style>
