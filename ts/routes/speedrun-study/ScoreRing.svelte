<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import {
        capitalize,
        displayNumber,
        displayRange,
        driverReasons,
        formatCoverage,
        gaugePercent,
        type ScoreEnvelope,
    } from "../speedrun-dashboard/lib";

    export let envelope: ScoreEnvelope;
    // The score's full name (Memory/Performance/Readiness): the accessible label
    // and the desktop eyebrow.
    export let name: string;
    // The compact eyebrow for the phone tile (MEMORY/PERFORM/READY).
    export let shortName: string;
    // What the coverage figure measures ("deck", "skills", "exam"): "covers X% of
    // <noun>".
    export let coverageNoun: string;
    // Short mono caption shown while the score is locked (e.g. "No reviews yet").
    export let lockedCaption: string;
    // The score's own hue + its lighter likely-range band, as CSS values.
    export let color: string;
    export let band: string;
    export let error: string | null = null;

    // A 270° gauge that opens at the bottom, matching the canvas geometry: r=46 in
    // a 120x120 box, so the visible sweep is three quarters of the circumference.
    const R = 46;
    const C = 2 * Math.PI * R;
    const SWEEP = C * 0.75;

    const clamp01 = (n: number): number => Math.max(0, Math.min(1, n));

    // A dial only draws a fill when there is a real number; otherwise it locks to
    // a track-only ring and shows what it still needs, never a fake reading.
    $: delivered = !error && !envelope.abstained;
    $: fmt = envelope.format;
    $: unit = fmt === "points" ? "" : "%";
    // The band runs from the start of the sweep up to the low bound; the brighter
    // marker caps it, spanning the likely range [low, high].
    $: lowFrac = clamp01(gaugePercent(envelope.rangeLow, fmt) / 100);
    $: highFrac = clamp01(gaugePercent(envelope.rangeHigh, fmt) / 100);
    $: bandLen = lowFrac * SWEEP;
    $: markerLen = Math.max(0, highFrac - lowFrac) * SWEEP;
    $: rangeText = `${displayNumber(envelope.rangeLow, fmt)}\u2013${displayNumber(envelope.rangeHigh, fmt)}`;
    $: rangeUnit = displayRange(envelope.rangeLow, envelope.rangeHigh, fmt);
    $: coverageText = `Covers ${formatCoverage(envelope.coveragePct)} of ${coverageNoun}`;
    $: coverageShort = `${formatCoverage(envelope.coveragePct)} ${coverageNoun}`;
    // At most two evidence lines, so the dial stays scannable.
    $: reasons = delivered ? driverReasons(envelope.reasons).slice(0, 2) : [];
    $: lockedReason = lockedReasonFor(error, envelope.abstainReason);

    function lockedReasonFor(err: string | null, abstainReason: string): string {
        if (err) {
            return "Couldn't load this score.";
        }
        return abstainReason
            ? capitalize(abstainReason)
            : "Study to unlock this score.";
    }
    $: label = delivered
        ? `${name}: likely ${rangeText}${unit}, ${coverageText.toLowerCase()}`
        : `${name}: no read yet. ${lockedReason}`;
</script>

<section
    class="card"
    class:locked={!delivered}
    style="--ring:{color};--band:{band}"
    role="img"
    aria-label={label}
>
    <!-- Desktop: the full gauge. -->
    <div class="dial-view">
        <div class="eyebrow">{name}</div>
        <div class="dial">
            <svg width="120" height="120" viewBox="0 0 120 120" aria-hidden="true">
                <circle
                    class="track"
                    cx="60"
                    cy="60"
                    r={R}
                    stroke-dasharray="{SWEEP} {C}"
                    transform="rotate(135 60 60)"
                />
                {#if delivered}
                    <circle
                        class="band"
                        cx="60"
                        cy="60"
                        r={R}
                        stroke-dasharray="{bandLen} {C}"
                        transform="rotate(135 60 60)"
                    />
                    <circle
                        class="marker"
                        cx="60"
                        cy="60"
                        r={R}
                        stroke-dasharray="{markerLen} {C}"
                        stroke-dashoffset={-bandLen}
                        transform="rotate(135 60 60)"
                    />
                {/if}
            </svg>
            <div class="center">
                {#if delivered}
                    <div class="value">
                        {rangeText}
                        <span class="unit">{unit}</span>
                    </div>
                    <div class="cap">Likely range</div>
                {:else}
                    <div class="value empty">&mdash;</div>
                    <div class="cap">Awaiting data</div>
                {/if}
            </div>
        </div>
        <div class="cover">{delivered ? coverageText : lockedCaption}</div>
        {#if !delivered}
            <div class="notes"><p class="note">{lockedReason}</p></div>
        {:else if reasons.length}
            <div class="notes">
                {#each reasons as reason}
                    <p class="note">{reason}</p>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Phone: a compact tile, no ring. -->
    <div class="tile">
        <div class="tile-name">{shortName}</div>
        <div class="tile-val" class:empty={!delivered}>
            {delivered ? rangeUnit : "\u2014"}
        </div>
        <div class="tile-sub">{delivered ? coverageShort : "no data"}</div>
    </div>
</section>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .card {
        min-width: 0;
        padding: 20px;
        @include syn.card;
    }

    .eyebrow {
        @include syn.eyebrow;
    }

    // ── Desktop gauge ────────────────────────────────────────────────────────
    .dial {
        @include syn.dial;
    }
    svg {
        display: block;
    }
    circle {
        fill: none;
        stroke-width: 10;
        stroke-linecap: round;
    }
    .track {
        stroke: var(--sr-track);
    }
    .band {
        stroke: var(--band);
    }
    .marker {
        stroke: var(--ring);
        transition: stroke-dasharray 0.5s ease;
    }

    .center {
        @include syn.dial-center;
    }
    .value {
        @include syn.dial-value;
        font-variant-numeric: tabular-nums;
    }
    .value .unit {
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0;
        color: var(--sr-ink-3);
    }
    .value.empty {
        font-size: 24px;
        color: var(--sr-faint);
    }
    .cap {
        @include syn.dial-caption;
        text-transform: uppercase;
    }
    .locked .cap {
        color: var(--sr-faint);
    }

    .cover {
        margin-top: 2px;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 10.5px;
        text-align: center;
        text-transform: uppercase;
        color: var(--sr-ink-slate);
    }

    .notes {
        margin-top: 12px;
        padding-top: 10px;
        border-top: 1px solid var(--sr-line-2);
    }
    .note {
        margin: 0;
        font-size: 11px;
        line-height: 1.55;
        color: var(--sr-ink-2);
    }
    .note + .note {
        margin-top: 4px;
    }

    // ── Phone tile ───────────────────────────────────────────────────────────
    .tile {
        display: none;
        padding: 1px 0;
        text-align: center;
    }
    .tile-name {
        font-family: var(--sr-mono);
        font-weight: 600;
        font-size: 8px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .tile-val {
        margin-top: 5px;
        font-size: 15px;
        font-weight: 700;
        color: var(--sr-ink);
        font-variant-numeric: tabular-nums;
    }
    .tile-val.empty {
        font-size: 17px;
        color: var(--sr-faint);
    }
    .tile-sub {
        margin-top: 3px;
        font-family: var(--sr-mono);
        font-size: 8px;
        color: var(--sr-faint);
    }

    // Below the phone breakpoint the ring gives way to the compact tile.
    @media (max-width: 40rem) {
        .card {
            padding: 11px 8px;
            border-radius: var(--sr-radius-node);
            box-shadow: var(--sr-shadow-phone);
        }
        .dial-view {
            display: none;
        }
        .tile {
            display: block;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .marker {
            transition: none;
        }
    }
</style>
