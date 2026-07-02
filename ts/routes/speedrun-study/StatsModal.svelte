<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { createEventDispatcher } from "svelte";

    import {
        deferredEnvelope,
        formatCoverage,
        type ScoreEnvelope,
    } from "../speedrun-dashboard/lib";
    import ScoreTile from "../speedrun-dashboard/ScoreTile.svelte";
    import type { SubjectBreakdown } from "./lib";

    export let open: boolean;
    export let memory: ScoreEnvelope | null;
    export let memoryError: string | null = null;
    export let performance: ScoreEnvelope | null;
    export let performanceError: string | null = null;
    export let readiness: ScoreEnvelope | null;
    export let readinessError: string | null = null;
    export let subjects: SubjectBreakdown;
    export let breakdownError: string | null = null;

    const dispatch = createEventDispatcher<{ close: void }>();
    const close = (): void => {
        dispatch("close");
    };

    const pct = (fraction: number): string => `${Math.round(fraction * 100)}%`;

    // Focus trap + Esc + restore focus, scoped to the dialog's lifetime.
    function trap(node: HTMLElement) {
        const previous = document.activeElement as HTMLElement | null;
        const focusables = (): HTMLElement[] =>
            Array.from(
                node.querySelectorAll<HTMLElement>(
                    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
                ),
            ).filter((el) => !el.hasAttribute("disabled"));

        function onKey(event: KeyboardEvent): void {
            if (event.key === "Escape") {
                event.preventDefault();
                close();
                return;
            }
            if (event.key !== "Tab") {
                return;
            }
            const items = focusables();
            if (items.length === 0) {
                return;
            }
            const first = items[0];
            const last = items[items.length - 1];
            if (event.shiftKey && document.activeElement === first) {
                event.preventDefault();
                last.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
                event.preventDefault();
                first.focus();
            }
        }

        node.addEventListener("keydown", onKey);
        (
            node.querySelector<HTMLElement>("[data-autofocus]") ??
            focusables()[0] ??
            node
        ).focus();

        return {
            destroy(): void {
                node.removeEventListener("keydown", onKey);
                previous?.focus?.();
            },
        };
    }
</script>

{#if open}
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="stats-shade" on:click={close}>
        <div
            class="stats-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="stats-title"
            tabindex="-1"
            use:trap
            on:click|stopPropagation
        >
            <header class="head">
                <h2 class="title" id="stats-title">Score details</h2>
                <button
                    class="stats-x"
                    data-autofocus
                    on:click={close}
                    aria-label="Close"
                >
                    &times;
                </button>
            </header>

            <div class="body">
                <section class="block">
                    <h3 class="block-head">Scores</h3>
                    <div class="instruments">
                        <ScoreTile
                            name="Memory"
                            question="Will you recall a fact you were taught, right now?"
                            live={true}
                            envelope={memory ?? deferredEnvelope("")}
                            error={memoryError}
                        />
                        <ScoreTile
                            name="Performance"
                            question="Will you get a new, exam-style question right?"
                            live={true}
                            envelope={performance ?? deferredEnvelope("")}
                            error={performanceError}
                        />
                        <ScoreTile
                            name="Readiness"
                            question="What would you score on the 472 to 528 scale today?"
                            live={true}
                            envelope={readiness ?? deferredEnvelope("")}
                            error={readinessError}
                        />
                    </div>
                </section>

                <section class="block">
                    <h3 class="block-head">By subject</h3>
                    {#if breakdownError}
                        <p class="note">
                            Couldn't load the subject breakdown. {breakdownError}
                        </p>
                    {:else if !subjects.hasData}
                        <p class="note">
                            No subject has graded reviews or practice attempts yet.
                            Study this deck to fill in the meters below.
                        </p>
                    {/if}
                    {#if subjects.subjects.length}
                        <ul class="subjects">
                            {#each subjects.subjects as s (s.id)}
                                <li class="subject">
                                    <div class="s-head">
                                        <span class="s-name">{s.name}</span>
                                        <span class="s-weight">
                                            {pct(s.examWeight)} of exam
                                        </span>
                                    </div>
                                    <div class="meters">
                                        <div class="meter">
                                            <div class="m-top">
                                                <span class="m-label">Coverage</span>
                                                <span class="m-val">
                                                    {formatCoverage(s.coverage)}
                                                </span>
                                            </div>
                                            <div class="m-track">
                                                <span
                                                    class="m-fill"
                                                    style="width:{s.coverage * 100}%"
                                                ></span>
                                            </div>
                                            <span class="m-sub">
                                                {s.topicCount} topics
                                            </span>
                                        </div>
                                        <div class="meter">
                                            <div class="m-top">
                                                <span class="m-label">
                                                    Retrievability
                                                </span>
                                                <span
                                                    class="m-val"
                                                    class:muted={!s.hasMemoryData}
                                                >
                                                    {s.hasMemoryData
                                                        ? pct(s.meanRetrievability)
                                                        : "no reviews"}
                                                </span>
                                            </div>
                                            <div class="m-track">
                                                <span
                                                    class="m-fill"
                                                    style="width:{s.hasMemoryData
                                                        ? s.meanRetrievability * 100
                                                        : 0}%"
                                                ></span>
                                            </div>
                                            <span class="m-sub">
                                                {s.memoryReviews} reviews
                                            </span>
                                        </div>
                                        <div class="meter">
                                            <div class="m-top">
                                                <span class="m-label">Application</span>
                                                <span
                                                    class="m-val"
                                                    class:muted={!s.hasApplicationData}
                                                >
                                                    {s.hasApplicationData
                                                        ? pct(s.applicationAccuracy)
                                                        : "not practiced"}
                                                </span>
                                            </div>
                                            <div class="m-track">
                                                <span
                                                    class="m-fill"
                                                    style="width:{s.hasApplicationData
                                                        ? s.applicationAccuracy * 100
                                                        : 0}%"
                                                ></span>
                                            </div>
                                            <span class="m-sub">
                                                {s.applicationAttempts} attempts
                                            </span>
                                        </div>
                                    </div>
                                </li>
                            {/each}
                        </ul>
                    {/if}
                </section>

                <section class="block">
                    <h3 class="block-head">What feeds each score</h3>
                    <dl class="feeds">
                        <div>
                            <dt>Memory</dt>
                            <dd>
                                Retrievability across your reviewed cards, and how much
                                of the map you have covered.
                            </dd>
                        </div>
                        <div>
                            <dt>Performance</dt>
                            <dd>
                                Your accuracy on graded, exam-style application
                                attempts, weighted by exam emphasis.
                            </dd>
                        </div>
                        <div>
                            <dt>Readiness</dt>
                            <dd>
                                Performance projected onto the 472 to 528 scale, with
                                the range widened by thin coverage.
                            </dd>
                        </div>
                    </dl>
                </section>
            </div>
        </div>
    </div>
{/if}

<style lang="scss">
    .stats-shade {
        position: fixed;
        inset: 0;
        z-index: 40;
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding: 2.5rem 1rem;
        overflow-y: auto;
        background: rgba(10, 12, 15, 0.5);
        animation: fade 0.14s ease;
    }
    .stats-dialog {
        width: 100%;
        max-width: 46rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        border: 1px solid var(--sr-line-2);
        border-radius: 10px;
        box-shadow: 0 24px 60px rgba(0, 0, 0, 0.28);
        animation: rise 0.16s ease;
    }

    .head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 1.1rem 1.35rem;
        border-bottom: 2px solid var(--sr-ink);
    }
    .title {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 700;
        letter-spacing: -0.015em;
    }
    .stats-x {
        appearance: none;
        border: 1px solid var(--sr-line-2);
        border-radius: 6px;
        width: 2rem;
        height: 2rem;
        font-size: 1.2rem;
        line-height: 1;
        background: none;
        color: var(--sr-ink-2);
        cursor: pointer;
    }
    .stats-x:hover {
        background: var(--sr-panel-2);
        color: var(--sr-ink);
    }

    .body {
        padding: 1.35rem;
    }
    .block {
        margin-top: 1.75rem;
    }
    .block:first-child {
        margin-top: 0;
    }
    .block-head {
        margin: 0 0 0.85rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .note {
        margin: 0 0 0.9rem;
        font-size: 0.88rem;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }

    .instruments {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        overflow: hidden;
    }

    .subjects {
        margin: 0;
        padding: 0;
        list-style: none;
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .subject {
        padding: 0.95rem 1.1rem;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 8px;
    }
    .s-head {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 0.75rem;
        margin-bottom: 0.8rem;
    }
    .s-name {
        font-size: 1rem;
        font-weight: 620;
    }
    .s-weight {
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }

    .meters {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 0.75rem 1.25rem;
    }
    .m-top {
        display: flex;
        justify-content: space-between;
        gap: 0.5rem;
        margin-bottom: 0.3rem;
    }
    .m-label {
        font-family: var(--sr-mono);
        font-size: 9.5px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .m-val {
        font-size: 0.85rem;
        font-weight: 620;
        font-variant-numeric: tabular-nums;
    }
    .m-val.muted {
        font-size: 0.72rem;
        font-weight: 500;
        color: var(--sr-ink-3);
    }
    .m-track {
        height: 6px;
        border-radius: 3px;
        background: var(--sr-panel-2);
        border: 1px solid var(--sr-line);
        overflow: hidden;
    }
    .m-fill {
        display: block;
        height: 100%;
        background: var(--sr-signal);
    }
    .m-sub {
        display: block;
        margin-top: 0.3rem;
        font-family: var(--sr-mono);
        font-size: 9px;
        letter-spacing: 0.04em;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }

    .feeds {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        margin: 0;
    }
    .feeds div {
        display: grid;
        grid-template-columns: 7rem 1fr;
        gap: 0.75rem;
    }
    .feeds dt {
        font-weight: 620;
        font-size: 0.9rem;
    }
    .feeds dd {
        margin: 0;
        font-size: 0.88rem;
        line-height: 1.45;
        color: var(--sr-ink-2);
    }

    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    @keyframes fade {
        from {
            opacity: 0;
        }
    }
    @keyframes rise {
        from {
            transform: translateY(8px);
            opacity: 0;
        }
    }

    @media (max-width: 40rem) {
        .instruments,
        .meters {
            grid-template-columns: 1fr;
        }
        .feeds div {
            grid-template-columns: 1fr;
            gap: 0.15rem;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .stats-shade,
        .stats-dialog {
            animation: none;
        }
    }
</style>
