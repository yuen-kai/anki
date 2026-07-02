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
        formatUpdated,
        gaugeBounds,
        gaugePercent,
        type ScoreEnvelope,
    } from "./lib";

    export let name: string;
    export let question: string;
    export let envelope: ScoreEnvelope;
    // Whether this score is computed live. Drives the abstain wording only:
    // a live score is "not enough data yet", a deferred one "not available yet".
    export let live: boolean;
    export let error: string | null = null;

    // The signal is spent only when the tile carries a real number. An abstaining
    // or errored tile stays quiet, so amber always means "here is a result".
    $: delivered = !error && !envelope.abstained;
    $: drivers = driverReasons(envelope.reasons);
    $: updated = formatUpdated(envelope.updatedAtSecs);
    $: unit = envelope.format === "points" ? "" : "%";

    // Gauge geometry: the marker at the estimate, the band across the likely range.
    $: mark = gaugePercent(envelope.estimate, envelope.format);
    $: bandLo = gaugePercent(envelope.rangeLow, envelope.format);
    $: bandHi = gaugePercent(envelope.rangeHigh, envelope.format);
    $: bounds = gaugeBounds(envelope.format);
</script>

<section class="ins" class:on={delivered} class:off={!delivered}>
    <div class="top">
        <span class="ins-name">{name}</span>
        {#if delivered}
            <span class="flag">Read</span>
        {:else}
            <span class="flag muted">No read</span>
        {/if}
    </div>
    <p class="question">{question}</p>

    {#if error}
        <p class="num empty">&mdash;</p>
        <p class="abstain">Couldn't load this score. {error}</p>
    {:else if envelope.abstained}
        <p class="num empty">&mdash;</p>
        <p class="abstain">
            {live ? "Not enough data yet." : "Not available yet."}
            {#if envelope.abstainReason}{capitalize(envelope.abstainReason)}{/if}
        </p>
    {:else}
        <p class="num">
            {displayNumber(envelope.estimate, envelope.format)}
            <span class="unit">{unit}</span>
        </p>

        <div class="gauge">
            <div class="track">
                <div
                    class="band"
                    style="left:{bandLo}%;width:{Math.max(0, bandHi - bandLo)}%"
                ></div>
                <div class="mark" style="left:{mark}%"></div>
            </div>
            <div class="scale">
                <span>{bounds[0]}</span>
                <span class="likely">
                    likely {displayRange(
                        envelope.rangeLow,
                        envelope.rangeHigh,
                        envelope.format,
                    )}
                </span>
                <span>{bounds[1]}</span>
            </div>
        </div>

        <dl class="meta">
            <div>
                <dt>Coverage</dt>
                <dd>{formatCoverage(envelope.coveragePct)}</dd>
            </div>
            <div>
                <dt>Confidence</dt>
                <dd>{capitalize(envelope.confidence)}</dd>
            </div>
        </dl>

        {#if drivers.length}
            <ul class="reasons">
                {#each drivers as reason}
                    <li>{reason}</li>
                {/each}
            </ul>
        {/if}

        {#if updated}
            <p class="stamp">Updated {updated}</p>
        {/if}
    {/if}
</section>

<style lang="scss">
    .ins {
        display: flex;
        flex-direction: column;
        min-width: 0;
        padding: 1.35rem 1.4rem 1.5rem;
        border-left: 1px solid var(--sr-line);
    }
    // Divider only between tiles; the grid clips the first one's border.
    :global(.instruments) > .ins:first-child {
        border-left: none;
    }
    // A quiet hatch marks an abstaining instrument as deliberately blank.
    .ins.off {
        background-image: repeating-linear-gradient(
            135deg,
            transparent,
            transparent 7px,
            var(--sr-hatch) 7px,
            var(--sr-hatch) 8px
        );
    }

    .top {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.5rem;
    }
    .ins-name {
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-ink);
    }
    .flag {
        font-family: var(--sr-mono);
        font-size: 9.5px;
        font-weight: 600;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        padding: 0.2rem 0.42rem;
        border-radius: 3px;
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
    }
    .flag.muted {
        background: transparent;
        color: var(--sr-ink-3);
        border: 1px solid var(--sr-line-2);
    }

    .question {
        margin: 0.55rem 0 0;
        font-size: 0.82rem;
        line-height: 1.4;
        color: var(--sr-ink-2);
        min-height: 2.3em;
    }

    .num {
        margin: 1.1rem 0 0;
        font-size: 3.1rem;
        font-weight: 730;
        line-height: 0.9;
        letter-spacing: -0.03em;
        font-variant-numeric: tabular-nums;
    }
    .num .unit {
        font-size: 1.35rem;
        font-weight: 600;
        color: var(--sr-ink-2);
        margin-left: 0.08em;
    }
    .num.empty {
        color: var(--sr-ink-3);
        font-weight: 600;
    }

    .gauge {
        margin-top: 1rem;
    }
    .track {
        position: relative;
        height: 10px;
        background: var(--sr-panel-2);
        border: 1px solid var(--sr-line);
        border-radius: 2px;
        overflow: hidden;
    }
    .band {
        position: absolute;
        top: 0;
        bottom: 0;
        background: var(--sr-signal-weak);
        border-left: 1px solid var(--sr-signal-line);
        border-right: 1px solid var(--sr-signal-line);
    }
    .mark {
        position: absolute;
        top: -3px;
        bottom: -3px;
        width: 2px;
        background: var(--sr-ink);
        transform: translateX(-1px);
    }
    .scale {
        display: flex;
        justify-content: space-between;
        gap: 0.5rem;
        margin-top: 0.35rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        color: var(--sr-ink-3);
    }
    .scale .likely {
        color: var(--sr-ink-2);
    }

    .meta {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 0.5rem 1rem;
        margin: 1rem 0 0;
        padding-top: 0.9rem;
        border-top: 1px solid var(--sr-line);
    }
    .meta dt {
        font-family: var(--sr-mono);
        font-size: 9.5px;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .meta dd {
        margin: 0.1rem 0 0;
        font-size: 0.9rem;
        font-weight: 560;
        font-variant-numeric: tabular-nums;
    }

    .reasons {
        margin: 0.95rem 0 0;
        padding: 0;
        list-style: none;
    }
    .reasons li {
        position: relative;
        padding-left: 0.85rem;
        margin-top: 0.25rem;
        font-size: 0.82rem;
        line-height: 1.45;
        color: var(--sr-ink-2);
    }
    .reasons li::before {
        content: "";
        position: absolute;
        left: 0;
        top: 0.55em;
        width: 4px;
        height: 4px;
        background: var(--sr-signal);
        border-radius: 1px;
    }

    .stamp {
        margin: 1rem 0 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.04em;
        color: var(--sr-ink-3);
    }

    .abstain {
        margin: 1.1rem 0 0;
        font-size: 0.85rem;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }
</style>
