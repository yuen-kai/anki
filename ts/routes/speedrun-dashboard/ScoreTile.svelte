<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import {
        capitalize,
        driverReasons,
        formatCoverage,
        formatEstimate,
        formatRange,
        formatUpdated,
        type ScoreEnvelope,
    } from "./lib";

    export let name: string;
    export let question: string;
    export let envelope: ScoreEnvelope;
    // Whether this score is computed live. Drives the abstain wording only:
    // a live score is "not enough data yet", a deferred one "not available yet".
    export let live: boolean;
    export let error: string | null = null;

    // The accent is spent only when the tile carries a real number. An abstaining
    // or errored tile stays quiet, so the teal always means "here is a result".
    $: delivered = !error && !envelope.abstained;
    $: drivers = driverReasons(envelope.reasons);
    $: updated = formatUpdated(envelope.updatedAtSecs);
</script>

<section class="tile" class:delivered>
    <header>
        <h3>{name}</h3>
        <p class="question">{question}</p>
    </header>

    {#if error}
        <div class="state">
            <p class="headline">Couldn't load this score</p>
            <p class="detail">{error}</p>
        </div>
    {:else if envelope.abstained}
        <div class="state">
            <p class="headline">{live ? "Not enough data yet" : "Not available yet"}</p>
            {#if envelope.abstainReason}
                <p class="detail">{capitalize(envelope.abstainReason)}</p>
            {/if}
        </div>
    {:else}
        <div class="value">
            <span class="estimate">
                {formatEstimate(envelope.estimate, envelope.format)}
            </span>
            <span class="range">
                likely {formatRange(
                    envelope.rangeLow,
                    envelope.rangeHigh,
                    envelope.format,
                )}
            </span>
        </div>

        <dl class="meta">
            <div class="stat">
                <dt>Coverage</dt>
                <dd>{formatCoverage(envelope.coveragePct)}</dd>
            </div>
            <div class="stat">
                <dt>Confidence</dt>
                <dd>{envelope.confidence}</dd>
            </div>
            <div class="stat">
                <dt>Graded reviews</dt>
                <dd>{envelope.gradedReviews}</dd>
            </div>
            <div class="stat">
                <dt>Updated</dt>
                <dd>{updated}</dd>
            </div>
        </dl>

        {#if drivers.length}
            <div class="reasons">
                <p class="reasons-label">Main reasons</p>
                <ul>
                    {#each drivers as reason}
                        <li>{reason}</li>
                    {/each}
                </ul>
            </div>
        {/if}
    {/if}
</section>

<style lang="scss">
    .tile {
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
        min-width: 0;
        padding: 1.2rem 1.3rem 1.35rem;
        background: var(--canvas-elevated);
        border: 1px solid var(--border-subtle);
        border-radius: 10px;
    }

    // The one accent on a score: a hairline along the top, clipped to the rounded
    // corner. Present only on a delivered tile.
    .tile.delivered {
        box-shadow: inset 0 2px 0 0 var(--sr-accent, var(--accent-card));
    }

    header {
        h3 {
            margin: 0;
            font-size: 1.05rem;
            font-weight: 640;
            line-height: 1.2;
            letter-spacing: -0.006em;
            border: none;
        }
        .question {
            margin: 0.25rem 0 0;
            max-width: 34ch;
            font-size: 0.82rem;
            line-height: 1.4;
            color: var(--fg-subtle);
        }
    }

    .value {
        display: flex;
        align-items: baseline;
        flex-wrap: wrap;
        gap: 0.1rem 0.6rem;
    }
    .estimate {
        font-size: 2.5rem;
        font-weight: 650;
        line-height: 1;
        letter-spacing: -0.02em;
        font-variant-numeric: tabular-nums;
    }
    .range {
        font-size: 0.9rem;
        color: var(--fg-subtle);
        font-variant-numeric: tabular-nums;
    }

    .meta {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.6rem 1rem;
        margin: 0;
        .stat dt {
            font-size: 0.75rem;
            color: var(--fg-subtle);
        }
        .stat dd {
            margin: 0.05rem 0 0;
            font-size: 0.95rem;
            font-variant-numeric: tabular-nums;
            overflow-wrap: anywhere;
        }
    }

    .reasons {
        .reasons-label {
            margin: 0 0 0.35rem;
            font-size: 0.7rem;
            font-weight: 700;
            letter-spacing: 0.08em;
            text-transform: uppercase;
            color: var(--fg-subtle);
        }
        ul {
            margin: 0;
            padding-inline-start: 1.1rem;
        }
        li {
            font-size: 0.9rem;
            line-height: 1.4;
            margin-bottom: 0.2rem;
        }
    }

    .state {
        .headline {
            margin: 0 0 0.4rem;
            font-size: 1rem;
            font-weight: 600;
        }
        .detail {
            margin: 0;
            max-width: 42ch;
            font-size: 0.9rem;
            line-height: 1.5;
            color: var(--fg-subtle);
        }
    }
</style>
