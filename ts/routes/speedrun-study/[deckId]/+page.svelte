<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { deferredEnvelope } from "../../speedrun-dashboard/lib";
    import ConceptTree from "../ConceptTree.svelte";
    import {
        deckLeafName,
        deckParentName,
        type OverviewAction,
        overviewAction,
        showDecks,
        startStudy,
        todayProgress,
    } from "../lib";
    import ScoreRing from "../ScoreRing.svelte";
    import StatsModal from "../StatsModal.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    let busy = false;
    let actionError: string | null = null;
    let menuOpen = false;
    let modalOpen = false;
    let menuWrap: HTMLElement;

    const message = (err: unknown): string =>
        err instanceof Error ? err.message : String(err);

    $: summary = data.summary;
    $: progress = todayProgress(summary);
    $: deckName = summary?.deckName ?? "";
    $: leaf = deckLeafName(deckName);
    $: parent = deckParentName(deckName);
    // The done state is only trustworthy when the counts actually loaded.
    $: allDone = !data.summaryError && progress.allDone;

    async function start(): Promise<void> {
        if (busy) {
            return;
        }
        busy = true;
        actionError = null;
        try {
            await startStudy(data.deckId);
            // On success Qt moves to the reviewer and tears this page down; if no
            // cards were due it reloads the screen, which resets `busy`.
        } catch (err) {
            actionError = message(err);
            busy = false;
        }
    }

    async function back(): Promise<void> {
        try {
            await showDecks();
        } catch (err) {
            actionError = message(err);
        }
    }

    async function runAction(action: OverviewAction): Promise<void> {
        menuOpen = false;
        try {
            await overviewAction(action);
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
        <div class="topbar">
            <button class="back" on:click={back}>
                <span class="chev" aria-hidden="true">&lsaquo;</span>
                Decks
            </button>
            <div class="menu-wrap" bind:this={menuWrap}>
                <button
                    class="more"
                    aria-haspopup="menu"
                    aria-expanded={menuOpen}
                    on:click={() => (menuOpen = !menuOpen)}
                >
                    More
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
                        {#if data.filtered}
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
        </div>

        <header class="masthead">
            <div class="who">
                {#if parent}<p class="crumb">{parent}</p>{/if}
                <h1 class="deck">{leaf || "Study"}</h1>
            </div>
            <p class="tag">Study</p>
        </header>

        {#if actionError}
            <p class="notice" role="alert">{actionError}</p>
        {/if}

        <section class="today" class:is-done={allDone}>
            {#if allDone}
                <div class="done">
                    <p class="done-title">
                        {progress.done > 0
                            ? "All done for today"
                            : "Nothing due right now"}
                    </p>
                    <p class="done-line">
                        New cards and reviews appear here as they come due.
                    </p>
                </div>
            {:else}
                <div class="today-main">
                    <div class="headline">
                        <span class="big">{progress.remaining}</span>
                        <span class="big-label">left to study today</span>
                    </div>
                    <button class="btn primary start" on:click={start} disabled={busy}>
                        {busy ? "Starting" : "Start"}
                    </button>
                </div>
                {#if data.summaryError}
                    <p class="soft">
                        Couldn't load today's counts. {data.summaryError}
                    </p>
                {/if}
            {/if}

            <div class="bar" aria-hidden="true">
                <span class="bar-fill" style="width:{progress.fraction * 100}%"></span>
            </div>
            <div class="bar-legend">
                <span>{progress.done} of {progress.total} done today</span>
                {#if !allDone && summary}
                    <span class="counts">
                        New {summary.new} · Learn {summary.learn} · Review {summary.review}
                    </span>
                {/if}
            </div>
        </section>

        <section class="section">
            <div class="sec-head">
                <h2 class="sec-title">Scores</h2>
                <button class="link" on:click={() => (modalOpen = true)}>
                    More details
                </button>
            </div>
            <div class="rings">
                <ScoreRing
                    name="Memory"
                    live={true}
                    envelope={data.memory ?? deferredEnvelope("")}
                    error={data.memoryError}
                />
                <ScoreRing
                    name="Performance"
                    live={true}
                    envelope={data.performance ?? deferredEnvelope("")}
                    error={data.performanceError}
                />
                <ScoreRing
                    name="Readiness"
                    live={true}
                    envelope={data.readiness ?? deferredEnvelope("")}
                    error={data.readinessError}
                />
            </div>
        </section>

        <section class="section">
            <div class="sec-head">
                <h2 class="sec-title">Concept tree</h2>
                <p class="sec-note">
                    Your map of the deck, each topic at its mastery stage.
                </p>
            </div>
            <div class="tree-panel">
                <ConceptTree tree={data.tree} error={data.treeError} />
            </div>
        </section>
    </div>

    <StatsModal
        open={modalOpen}
        memory={data.memory}
        memoryError={data.memoryError}
        performance={data.performance}
        performanceError={data.performanceError}
        readiness={data.readiness}
        readinessError={data.readinessError}
        subjects={data.subjects}
        breakdownError={data.breakdownError}
        on:close={() => (modalOpen = false)}
    />
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    .study {
        @include sr.tokens;
        // ScoreTile's abstain hatch, not part of the shared token mixin.
        --sr-hatch: rgba(0, 0, 0, 0.014);

        box-sizing: border-box;
        min-height: 100%;
        padding: 1.5rem 1.75rem 3rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;
    }
    :global(.night-mode) .study {
        --sr-hatch: rgba(255, 255, 255, 0.02);
    }

    .inner {
        max-width: 56rem;
        margin: 0 auto;
    }

    .topbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        margin-bottom: 1.25rem;
    }
    .back {
        display: inline-flex;
        align-items: center;
        gap: 0.3rem;
        padding: 0.35rem 0.1rem;
        border: none;
        background: none;
        color: var(--sr-ink-2);
        font: inherit;
        font-size: 0.9rem;
        cursor: pointer;
    }
    .back:hover {
        color: var(--sr-ink);
        text-decoration: underline;
        text-underline-offset: 3px;
    }
    .chev {
        font-size: 1.2em;
        line-height: 0;
    }

    .menu-wrap {
        position: relative;
    }
    .more {
        appearance: none;
        border: 1px solid var(--sr-line-2);
        border-radius: 6px;
        padding: 0.4rem 0.8rem;
        background: none;
        color: var(--sr-ink-2);
        font: inherit;
        font-size: 0.85rem;
        font-weight: 560;
        cursor: pointer;
    }
    .more:hover {
        background: var(--sr-panel-2);
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
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        box-shadow: 0 14px 34px rgba(0, 0, 0, 0.18);
    }
    .more-menu button {
        display: block;
        width: 100%;
        padding: 0.5rem 0.65rem;
        border: none;
        border-radius: 5px;
        background: none;
        color: var(--sr-ink);
        font: inherit;
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

    .masthead {
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        gap: 1rem;
        padding-bottom: 0.9rem;
        border-bottom: 2px solid var(--sr-ink);
    }
    .crumb {
        margin: 0 0 0.15rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .deck {
        margin: 0;
        font-size: 1.7rem;
        font-weight: 720;
        letter-spacing: -0.02em;
        line-height: 1.1;
    }
    .tag {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.18em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }

    .notice {
        margin: 1.1rem 0 0;
        padding: 0.7rem 0.9rem;
        border: 1px solid var(--sr-line-2);
        border-left: 3px solid var(--sr-ink-3);
        border-radius: 6px;
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        font-size: 0.9rem;
    }

    // Today's Progress: the remaining count leads, Start sits beside it, the bar
    // reads left to right toward done.
    .today {
        margin-top: 1.6rem;
        padding: 1.35rem 1.45rem 1.4rem;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 10px;
    }
    .today-main {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1.25rem;
    }
    .headline {
        display: flex;
        align-items: baseline;
        gap: 0.6rem;
        min-width: 0;
    }
    .big {
        font-size: 3.4rem;
        font-weight: 730;
        line-height: 0.85;
        letter-spacing: -0.03em;
        font-variant-numeric: tabular-nums;
    }
    .big-label {
        font-size: 0.95rem;
        color: var(--sr-ink-2);
    }

    .done {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }
    .done-title {
        margin: 0;
        font-size: 1.35rem;
        font-weight: 680;
        letter-spacing: -0.015em;
    }
    .done-line {
        margin: 0;
        font-size: 0.9rem;
        color: var(--sr-ink-2);
    }

    .soft {
        margin: 0.75rem 0 0;
        font-size: 0.82rem;
        color: var(--sr-ink-3);
    }

    .bar {
        position: relative;
        height: 10px;
        margin-top: 1.25rem;
        background: var(--sr-panel-2);
        border: 1px solid var(--sr-line);
        border-radius: 3px;
        overflow: hidden;
    }
    .bar-fill {
        display: block;
        height: 100%;
        background: var(--sr-signal);
        transition: width 0.4s ease;
    }
    .bar-legend {
        display: flex;
        justify-content: space-between;
        gap: 0.75rem;
        margin-top: 0.5rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        letter-spacing: 0.04em;
        color: var(--sr-ink-3);
        font-variant-numeric: tabular-nums;
    }

    .section {
        margin-top: 2.5rem;
    }
    .sec-head {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 0.9rem;
        flex-wrap: wrap;
        margin-bottom: 1.35rem;
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--sr-line);
    }
    .sec-title {
        margin: 0;
        font-size: 1.35rem;
        font-weight: 680;
        letter-spacing: -0.015em;
        line-height: 1.1;
    }
    .sec-note {
        margin: 0;
        font-size: 0.85rem;
        color: var(--sr-ink-2);
    }
    .link {
        border: none;
        background: none;
        padding: 0;
        color: var(--sr-ink-2);
        font: inherit;
        font-size: 0.85rem;
        font-weight: 560;
        cursor: pointer;
        border-bottom: 1px solid var(--sr-line-2);
    }
    .link:hover {
        color: var(--sr-ink);
        border-bottom-color: var(--sr-signal);
    }

    .rings {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1rem;
        justify-items: center;
        padding: 0.5rem 0;
    }

    .tree-panel {
        padding: 1.4rem 1rem 1.5rem;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 10px;
    }

    .btn {
        appearance: none;
        border: 1px solid transparent;
        border-radius: 6px;
        padding: 0.6rem 1.4rem;
        font-family: var(--sr-sans);
        font-size: 0.95rem;
        font-weight: 600;
        line-height: 1;
        cursor: pointer;
        transition:
            background 0.12s ease,
            filter 0.12s ease;
    }
    .btn:disabled {
        opacity: 0.55;
        cursor: default;
    }
    .btn.primary {
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
    }
    .btn.primary:hover:not(:disabled) {
        filter: brightness(1.04);
    }
    .start {
        flex: 0 0 auto;
    }

    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    @media (max-width: 46rem) {
        .rings {
            grid-template-columns: 1fr;
            gap: 1.75rem;
        }
    }

    @media (max-width: 34rem) {
        .study {
            padding: 1.25rem 1rem 2.5rem;
        }
        .today-main {
            flex-direction: column;
            align-items: stretch;
            gap: 1rem;
        }
        .start {
            width: 100%;
        }
        .bar-legend {
            flex-direction: column;
            gap: 0.2rem;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .bar-fill {
            transition: none;
        }
    }
</style>
