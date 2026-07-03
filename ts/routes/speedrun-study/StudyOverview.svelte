<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The per-deck study overview surface (canvas 2b / not-started 2i): the Today bar,
the three Memory / Performance / Readiness dials, the concept tree, and the
per-subject table. Purely presentational — the host supplies the loaded data and
the start / back / overflow-action handlers, so both the real route (backed by
RPCs) and the backend-free demo render the same screen.
-->
<script lang="ts">
    import { deferredEnvelope, type ScoreEnvelope } from "../speedrun-dashboard/lib";
    import { isMobileShell } from "../speedrun-hierarchy/lib";
    import ConceptTree from "./ConceptTree.svelte";
    import {
        bottleneckSubject,
        type ConceptTreeNode,
        deckLeafName,
        type OverviewAction,
        type StudySummary,
        type SubjectBreakdown,
        todayProgress,
    } from "./lib";
    import ScoreRing from "./ScoreRing.svelte";
    import SubjectTable from "./SubjectTable.svelte";

    export let summary: StudySummary | null;
    export let summaryError: string | null = null;
    export let memory: ScoreEnvelope | null = null;
    export let memoryError: string | null = null;
    export let performance: ScoreEnvelope | null = null;
    export let performanceError: string | null = null;
    export let readiness: ScoreEnvelope | null = null;
    export let readinessError: string | null = null;
    export let tree: ConceptTreeNode | null = null;
    export let treeError: string | null = null;
    export let subjects: SubjectBreakdown;
    export let breakdownError: string | null = null;
    export let filtered = false;
    export let onStart: () => Promise<void> | void;
    export let onBack: () => Promise<void> | void;
    export let onAction: (action: OverviewAction) => Promise<void> | void;
    // The overflow actions open Qt dialogs, so the menu is desktop-only; the
    // mobile shell hides it.
    export let showMenu: boolean = !isMobileShell();

    let busy = false;
    let actionError: string | null = null;
    let menuOpen = false;
    let menuWrap: HTMLElement;

    const message = (err: unknown): string =>
        err instanceof Error ? err.message : String(err);

    $: prog = todayProgress(summary);
    $: leaf = deckLeafName(summary?.deckName ?? "");
    // No graded evidence in any score: the deck reads as "not started".
    $: notStarted = [memory, performance, readiness].every(
        (envelope) => !envelope || envelope.abstained,
    );
    $: topics = tree?.leafCount ?? 0;
    $: concepts = tree?.conceptCount ?? 0;
    $: metaText = metaTextFor(tree, topics, concepts, notStarted);

    function metaTextFor(
        node: ConceptTreeNode | null,
        topicCount: number,
        conceptCount: number,
        deckNotStarted: boolean,
    ): string {
        if (!node) {
            return deckNotStarted ? "Not started" : "";
        }
        const t = `${topicCount} ${topicCount === 1 ? "topic" : "topics"}`;
        const c = `${conceptCount} ${conceptCount === 1 ? "concept" : "concepts"}`;
        return `${t} · ${c}`;
    }
    // The done state is only trustworthy when the counts loaded; Start is
    // disabled whenever nothing is due (the not-started edge included).
    $: allDone = !summaryError && prog.allDone;
    $: bottleneckId = bottleneckSubject(subjects);

    async function start(): Promise<void> {
        if (busy) {
            return;
        }
        busy = true;
        actionError = null;
        try {
            await onStart();
            // On success Qt tears this page down; if nothing was due it reloads,
            // which resets `busy`.
        } catch (err) {
            actionError = message(err);
            busy = false;
        }
    }

    async function back(): Promise<void> {
        try {
            await onBack();
        } catch (err) {
            actionError = message(err);
        }
    }

    async function runAction(action: OverviewAction): Promise<void> {
        menuOpen = false;
        try {
            await onAction(action);
        } catch (err) {
            actionError = message(err);
        }
    }

    function onWindowClick(event: MouseEvent): void {
        if (menuOpen && menuWrap && !menuWrap.contains(event.target as HTMLElement)) {
            menuOpen = false;
        }
    }

    function onWindowKey(event: KeyboardEvent): void {
        if (event.key === "Escape") {
            menuOpen = false;
        }
    }
</script>

<svelte:window on:click={onWindowClick} on:keydown={onWindowKey} />

<div class="study">
    <div class="inner">
        <header class="hdr">
            <div class="hdr-left">
                <button class="back" on:click={back} aria-label="Back to decks">
                    <span aria-hidden="true">&larr;</span>
                     Decks
                </button>
                <span class="deck-name">{leaf || "Study"}</span>
                {#if metaText}<span class="deck-meta">{metaText}</span>{/if}
            </div>
            {#if showMenu}
                <div class="menu-wrap" bind:this={menuWrap}>
                    <button
                        class="opts"
                        aria-label="Options"
                        aria-haspopup="menu"
                        aria-expanded={menuOpen}
                        on:click={() => (menuOpen = !menuOpen)}
                    >
                        ⋯
                    </button>
                    {#if menuOpen}
                        <ul class="more-menu" role="menu">
                            <li role="none">
                                <button
                                    role="menuitem"
                                    on:click={() => runAction("options")}
                                >
                                    Deck options
                                </button>
                            </li>
                            <li role="none">
                                <button
                                    role="menuitem"
                                    on:click={() => runAction("customStudy")}
                                >
                                    Custom study
                                </button>
                            </li>
                            <li role="none">
                                <button
                                    role="menuitem"
                                    on:click={() => runAction("unbury")}
                                >
                                    Unbury
                                </button>
                            </li>
                            <li role="none">
                                <button
                                    role="menuitem"
                                    on:click={() => runAction("description")}
                                >
                                    Edit description
                                </button>
                            </li>
                            {#if filtered}
                                <li class="sep" role="none">
                                    <button
                                        role="menuitem"
                                        on:click={() => runAction("rebuild")}
                                    >
                                        Rebuild
                                    </button>
                                </li>
                                <li role="none">
                                    <button
                                        role="menuitem"
                                        on:click={() => runAction("empty")}
                                    >
                                        Empty
                                    </button>
                                </li>
                            {/if}
                        </ul>
                    {/if}
                </div>
            {/if}
        </header>

        {#if actionError}
            <p class="notice" role="alert">{actionError}</p>
        {/if}

        <section class="today">
            <div class="today-count">
                <div class="eyebrow">Today</div>
                {#if summaryError}
                    <div class="due-none">Couldn't load today's counts</div>
                {:else if allDone}
                    <div class="due-none">
                        {prog.done > 0 ? "All done for today" : "Nothing due yet"}
                    </div>
                {:else}
                    <div class="due">
                        <span class="due-num">{prog.remaining}</span>
                        <span class="due-word">cards left</span>
                    </div>
                {/if}
            </div>

            {#if !allDone && !summaryError && prog.total > 0}
                <div class="today-progress">
                    <div class="bar">
                        <span
                            class="bar-fill"
                            style="width:{prog.fraction * 100}%"
                        ></span>
                    </div>
                    <div class="progress-note">
                        {prog.done} of {prog.total} done today
                    </div>
                </div>
            {/if}

            <div class="today-cta">
                <button
                    class="start"
                    class:off={busy || allDone}
                    on:click={start}
                    disabled={busy || allDone}
                >
                    {busy ? "Starting" : "Start studying"}
                </button>
            </div>
        </section>

        <section class="dials">
            <ScoreRing
                name="Memory"
                shortName="MEMORY"
                coverageNoun="deck"
                lockedCaption="No reviews yet"
                color="var(--sr-score-memory)"
                band="var(--sr-score-memory-band)"
                envelope={memory ?? deferredEnvelope("")}
                error={memoryError}
            />
            <ScoreRing
                name="Performance"
                shortName="PERFORM"
                coverageNoun="skills"
                lockedCaption="No graded work yet"
                color="var(--sr-score-performance)"
                band="var(--sr-score-performance-band)"
                envelope={performance ?? deferredEnvelope("")}
                error={performanceError}
            />
            <ScoreRing
                name="Readiness"
                shortName="READY"
                coverageNoun="exam"
                lockedCaption="Not enough data yet"
                color="var(--sr-score-readiness)"
                band="var(--sr-score-readiness-band)"
                envelope={readiness ?? deferredEnvelope("")}
                error={readinessError}
            />
        </section>

        {#if tree || treeError}
            <section class="tree-card">
                <ConceptTree {tree} error={treeError} />
            </section>
        {/if}

        {#if subjects.subjects.length || breakdownError}
            <section class="subjects-card">
                <SubjectTable {subjects} error={breakdownError} {bottleneckId} />
            </section>
        {/if}
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;
    @use "$lib/sass/speedrun-synapse" as syn;

    .study {
        box-sizing: border-box;
        min-height: 100%;
        padding-bottom: 38px;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        -webkit-font-smoothing: antialiased;
        @include sr.tokens;
    }
    .inner {
        // Matches the 2b canvas frame width; cards inset 40px, so full-width
        // cards run 1160px and the fixed 1040px concept tree fits with slack.
        max-width: 1240px;
        margin: 0 auto;
    }

    // ── Header ───────────────────────────────────────────────────────────────
    .hdr {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding: 16px 40px;
        border-bottom: 1px solid var(--sr-line-2);
    }
    .hdr-left {
        display: flex;
        align-items: center;
        gap: 14px;
        min-width: 0;
    }
    .back {
        flex-shrink: 0;
        padding: 0;
        border: none;
        background: none;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 12px;
        color: var(--sr-ink-3);
        cursor: pointer;
    }
    .back:hover {
        color: var(--sr-ink);
    }
    .deck-name {
        font-size: 17px;
        font-weight: 700;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .deck-meta {
        flex-shrink: 0;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 10px;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }

    .menu-wrap {
        position: relative;
        flex-shrink: 0;
    }
    .opts {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 34px;
        height: 34px;
        padding: 0;
        border: none;
        border-radius: var(--sr-radius-control);
        background: var(--sr-ghost);
        color: var(--sr-ink-slate);
        font-size: 18px;
        font-weight: 700;
        line-height: 1;
        cursor: pointer;
    }
    .opts:hover {
        color: var(--sr-ink);
    }
    .more-menu {
        position: absolute;
        top: calc(100% + 0.4rem);
        right: 0;
        z-index: 20;
        min-width: 11rem;
        margin: 0;
        padding: 0.3rem;
        list-style: none;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-control);
        box-shadow: var(--sr-shadow-card);
    }
    .more-menu button {
        display: block;
        width: 100%;
        padding: 0.5rem 0.65rem;
        border: none;
        border-radius: var(--sr-radius-sm);
        background: none;
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 0.88rem;
        text-align: left;
        cursor: pointer;
    }
    .more-menu button:hover {
        background: var(--sr-panel-2);
    }
    .more-menu .sep {
        margin-top: 0.3rem;
        padding-top: 0.3rem;
        border-top: 1px solid var(--sr-line);
    }

    .notice {
        margin: 16px 40px 0;
        padding: 0.7rem 0.9rem;
        border: 1px solid var(--sr-line);
        border-left: 3px solid var(--sr-signal);
        border-radius: var(--sr-radius-control);
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        font-size: 0.9rem;
    }

    // ── Today ────────────────────────────────────────────────────────────────
    .today {
        display: flex;
        align-items: center;
        gap: 28px;
        margin: 24px 40px 0;
        padding: 20px 24px;
        @include syn.card;
    }
    .eyebrow {
        @include syn.eyebrow;
    }
    .today-count {
        flex-shrink: 0;
    }
    .due {
        margin-top: 2px;
    }
    .due-num {
        font-size: 34px;
        font-weight: 700;
        letter-spacing: -0.02em;
        font-variant-numeric: tabular-nums;
    }
    .due-word {
        font-size: 14px;
        font-weight: 500;
        color: var(--sr-ink-3);
    }
    .due-none {
        margin-top: 2px;
        font-size: 22px;
        font-weight: 700;
    }
    .today-progress {
        flex: 1;
        min-width: 0;
    }
    .bar {
        @include syn.bar;
        height: 10px;
        border-radius: 6px;
    }
    .bar-fill {
        @include syn.bar-fill;
        border-radius: 6px;
    }
    .progress-note {
        margin-top: 8px;
        font-family: var(--sr-mono);
        font-size: 11px;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }
    .today-cta {
        margin-left: auto;
        flex-shrink: 0;
    }
    .start {
        @include syn.btn;
        @include syn.btn-primary;
        @include syn.btn-hero;
        box-shadow: var(--sr-shadow-signal);
    }
    .start:hover:not(.off) {
        filter: brightness(1.04);
    }
    .start.off {
        @include syn.btn-disabled;
    }

    // ── Score dials ──────────────────────────────────────────────────────────
    .dials {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 16px;
        margin: 16px 40px 0;
    }

    // ── Concept tree + subjects cards ────────────────────────────────────────
    .tree-card {
        margin: 16px 40px 0;
        padding: 24px 24px 30px;
        @include syn.card;
    }
    .subjects-card {
        margin: 16px 40px 0;
        padding: 20px 24px;
        @include syn.card;
    }

    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    // ── Phone (2f): tighter insets, header stacks, hit targets clear 44px ──────
    @media (max-width: 40rem) {
        .hdr {
            flex-wrap: wrap;
            gap: 6px 14px;
            padding: 12px 16px 10px;
        }
        .hdr-left {
            flex-wrap: wrap;
            gap: 2px 14px;
        }
        .back {
            order: -1;
            min-height: 44px;
            display: inline-flex;
            align-items: center;
        }
        .deck-name {
            flex-basis: 100%;
            font-size: 18px;
        }
        .deck-meta {
            display: none;
        }
        .notice {
            margin: 12px 16px 0;
        }
        .today {
            gap: 12px;
            margin: 12px 14px 0;
            padding: 14px 15px;
            border-radius: var(--sr-radius-card-sm);
            box-shadow: var(--sr-shadow-phone);
        }
        // Stack the count like 2f: big number over a small unit line.
        .due {
            display: flex;
            flex-direction: column;
            line-height: 1;
        }
        .due-num {
            font-size: 26px;
        }
        .due-word {
            margin-top: 2px;
            font-size: 9px;
        }
        .start {
            padding: 11px 16px;
            font-size: 13px;
            min-height: 44px;
        }
        .dials {
            gap: 8px;
            margin: 11px 14px 0;
        }
        .tree-card {
            margin: 11px 14px 0;
            padding: 13px 15px;
            border-radius: var(--sr-radius-card-sm);
            box-shadow: var(--sr-shadow-phone);
        }
        .subjects-card {
            margin: 11px 14px 0;
            padding: 14px 15px;
            border-radius: var(--sr-radius-card-sm);
            box-shadow: var(--sr-shadow-phone);
        }
    }
</style>
