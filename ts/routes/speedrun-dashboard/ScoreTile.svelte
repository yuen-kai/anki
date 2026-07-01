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
    // Memory is computed live; Performance and Readiness are deferred placeholders.
    export let live: boolean;
    export let error: string | null = null;

    $: drivers = driverReasons(envelope.reasons);
    $: updated = formatUpdated(envelope.updatedAtSecs);
</script>

<section class="tile" class:live class:deferred={!live}>
    <header>
        <h2>{name}</h2>
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
            <span class="estimate">{formatEstimate(envelope.estimate)}</span>
            <span class="range">
                likely {formatRange(envelope.rangeLow, envelope.rangeHigh)}
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
        gap: 0.85rem;
        min-width: 0;
        padding: 1.2rem 1.35rem 1.35rem;
        background: var(--canvas-elevated);
        border: 1px solid var(--border-subtle);
        border-radius: var(--border-radius-medium, 12px);
    }

    // The one accent, spent only on the live score: a hairline along the top.
    .tile.live {
        box-shadow: inset 0 3px 0 0 var(--sr-accent, var(--accent-card));
    }

    // Deferred scores recede: same card, quieter text, no accent.
    .tile.deferred {
        color: var(--fg-subtle);
    }

    header {
        h2 {
            margin: 0;
            font-size: 1.05rem;
            font-weight: 600;
            border: none;
        }
        .question {
            margin: 0.2rem 0 0;
            font-size: 0.85rem;
            line-height: 1.35;
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
        font-size: 2.6rem;
        font-weight: 650;
        line-height: 1;
        letter-spacing: -0.01em;
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
        gap: 0.55rem 1rem;
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
            margin: 0 0 0.3rem;
            font-size: 0.75rem;
            color: var(--fg-subtle);
        }
        ul {
            margin: 0;
            padding-inline-start: 1.1rem;
        }
        li {
            font-size: 0.9rem;
            margin-bottom: 0.15rem;
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
