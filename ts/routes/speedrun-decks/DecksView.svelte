<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The Decks home surface (canvas 2a / empty 2h): brand header, deck rows with
completion bars and Study / Details actions, the dashed new-deck affordance, and
the empty state. Purely presentational — the host supplies the deck data and the
study/create/details/delete handlers, so both the real route (backed by RPCs)
and the backend-free demo render the same screen.
-->
<script lang="ts">
    import type { DeckRow } from "./lib";

    export let decks: DeckRow[];
    export let loadError: string | null = null;
    export let onStudy: (deck: DeckRow) => Promise<void> | void;
    export let onDetails: (deck: DeckRow) => void;
    export let onCreate: () => void;
    export let onDelete: (deck: DeckRow) => Promise<void> | void;

    let rows: DeckRow[] = decks;
    let busy = false;
    // The deck whose delete popover is open, if any.
    let menuId: string | null = null;

    $: isEmpty = rows.length === 0 && !loadError;

    // Header meta slot. The canvas shows an exam countdown here; nothing in the
    // backend supplies an exam date, so we show today's date instead — real,
    // neutral, no invented "days to exam". Built from parts to match the
    // canvas's comma-less "TUE JUL 1" style.
    const now = new Date();
    const today =
        `${now.toLocaleDateString("en-US", { weekday: "short" })} ${now.toLocaleDateString(
            "en-US",
            { month: "short" },
        )} ${now.getDate()}`.toUpperCase();

    const message = (err: unknown): string =>
        err instanceof Error ? err.message : String(err);

    async function study(deck: DeckRow): Promise<void> {
        if (busy) {
            return;
        }
        busy = true;
        try {
            await onStudy(deck);
        } catch (err) {
            loadError = message(err);
        } finally {
            busy = false;
        }
    }

    async function remove(deck: DeckRow): Promise<void> {
        busy = true;
        try {
            await onDelete(deck);
            rows = rows.filter((row) => row.deckId !== deck.deckId);
            menuId = null;
        } catch (err) {
            loadError = message(err);
        } finally {
            busy = false;
        }
    }

    const plural = (n: number, word: string): string =>
        `${n} ${word}${n === 1 ? "" : "s"}`;
    const percent = (fraction: number): number => Math.round(fraction * 100);
</script>

<svelte:window
    on:keydown={(event) => {
        if (event.key === "Escape") {
            menuId = null;
        }
    }}
/>

<div class="screen">
    <div class="wrap">
        <header class="hdr" class:hdr--slim={isEmpty}>
            <div class="brand">
                <span class="sr-logo" aria-hidden="true"></span>
                <span class="brand-name">Synapse</span>
                <span class="brand-tag">MCAT</span>
            </div>
            <div class="hdr-right">
                <span class="hdr-date">{today}</span>
                <span class="avatar" aria-hidden="true">
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <circle cx="12" cy="9" r="3.4" />
                        <path d="M5.5 19.5c.5-3.5 3.2-5.4 6.5-5.4s6 1.9 6.5 5.4z" />
                    </svg>
                </span>
            </div>
        </header>

        {#if loadError}
            <p class="notice" role="alert">Could not load decks: {loadError}</p>
        {/if}

        {#if isEmpty}
            <section class="empty">
                <div class="empty-mark" aria-hidden="true">+</div>
                <h1 class="empty-title">No decks yet</h1>
                <p class="empty-help">
                    Create your first deck and build its topic tree to start studying.
                </p>
                <button
                    type="button"
                    class="sr-btn sr-btn--primary sr-btn--hero"
                    on:click={onCreate}
                >
                    + New deck
                </button>
            </section>
        {:else if rows.length > 0}
            <div class="title-row">
                <div class="title-block">
                    <div class="sr-eyebrow">Decks</div>
                    <h1 class="title">Your decks</h1>
                </div>
                <button
                    type="button"
                    class="sr-btn sr-btn--primary sr-btn--new"
                    on:click={onCreate}
                >
                    + New deck
                </button>
            </div>

            <div class="rows">
                {#each rows as deck (deck.deckId)}
                    {@const m = deck.metrics}
                    <article class="sr-card row">
                        <div class="row-info">
                            <h2 class="row-name">{deck.name || "Untitled deck"}</h2>
                            <p class="row-meta">
                                {#if m}
                                    {plural(m.topics, "topic")} · {plural(
                                        m.concepts,
                                        "concept",
                                    )}
                                {:else}
                                    counts unavailable
                                {/if}
                            </p>
                        </div>

                        <div class="row-progress">
                            <div class="row-progress-head">
                                <span class="prog-label">Completion</span>
                                <span class="prog-value">
                                    {m ? `${percent(m.completion)}%` : "—"}
                                </span>
                            </div>
                            <div class="sr-bar" aria-hidden="true">
                                <div
                                    class="sr-bar__fill"
                                    style="width:{m ? percent(m.completion) : 0}%"
                                ></div>
                            </div>
                        </div>

                        <div class="row-actions">
                            <button
                                type="button"
                                class="sr-btn sr-btn--dark"
                                on:click={() => study(deck)}
                                disabled={busy}
                            >
                                Study
                            </button>
                            <button
                                type="button"
                                class="sr-btn sr-btn--ghost"
                                on:click={() => onDetails(deck)}
                                disabled={busy}
                            >
                                Details
                            </button>
                            <div class="menu-wrap">
                                <button
                                    type="button"
                                    class="more"
                                    aria-label="Deck options"
                                    aria-haspopup="true"
                                    aria-expanded={menuId === deck.deckId}
                                    on:click={() =>
                                        (menuId =
                                            menuId === deck.deckId
                                                ? null
                                                : deck.deckId)}
                                >
                                    <svg
                                        viewBox="0 0 24 24"
                                        width="18"
                                        height="18"
                                        fill="currentColor"
                                    >
                                        <circle cx="5" cy="12" r="1.6" />
                                        <circle cx="12" cy="12" r="1.6" />
                                        <circle cx="19" cy="12" r="1.6" />
                                    </svg>
                                </button>
                                {#if menuId === deck.deckId}
                                    <div class="menu">
                                        <p class="menu-q">Delete this deck?</p>
                                        <div class="menu-actions">
                                            <button
                                                type="button"
                                                class="sr-btn sr-btn--dark"
                                                on:click={() => remove(deck)}
                                                disabled={busy}
                                            >
                                                Delete
                                            </button>
                                            <button
                                                type="button"
                                                class="sr-btn sr-btn--ghost"
                                                on:click={() => (menuId = null)}
                                                disabled={busy}
                                            >
                                                Cancel
                                            </button>
                                        </div>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    </article>
                {/each}

                <button type="button" class="new-deck" on:click={onCreate}>
                    <span class="new-deck-plus" aria-hidden="true">+</span>
                    <span class="new-deck-text">New deck</span>
                </button>
            </div>
        {/if}
    </div>
</div>

{#if menuId !== null}
    <button
        type="button"
        class="backdrop"
        aria-label="Close menu"
        on:click={() => (menuId = null)}
    ></button>
{/if}

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    $phone: 40rem;

    .screen {
        box-sizing: border-box;
        min-height: 100%;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.5;
        -webkit-font-smoothing: antialiased;
        @include syn.host; // tokens + .sr-* classes; keep LAST
    }

    .wrap {
        max-width: 1240px;
        margin: 0 auto;
        padding-bottom: 40px;
    }

    // ── Header: brand mark + wordmark, date, avatar ──────────────────────────
    .hdr {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 18px 40px;
        border-bottom: 1px solid var(--sr-line-2);
    }
    .brand {
        display: flex;
        align-items: center;
        gap: 10px;
    }
    .brand-name {
        font-weight: 700;
        font-size: 16px;
        letter-spacing: -0.01em;
        color: var(--sr-ink);
    }
    .brand-tag {
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 10px;
        letter-spacing: 0.12em;
        color: var(--sr-ink-3);
        margin-left: 4px;
    }
    .hdr-right {
        display: flex;
        align-items: center;
        gap: 16px;
    }
    .hdr-date {
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 10px;
        letter-spacing: 0.12em;
        color: var(--sr-ink-3);
    }
    // No user identity to key an avatar off, so a neutral person glyph stands in
    // for the canvas's initials.
    .avatar {
        display: grid;
        place-items: center;
        width: 32px;
        height: 32px;
        border-radius: 50%;
        background: var(--sr-track);
        color: var(--sr-ink-slate);
    }

    // Slim header for the empty state (2h): brand only.
    .hdr--slim {
        padding: 18px 32px;
    }
    .hdr--slim .sr-logo {
        width: 24px;
        height: 24px;
    }
    .hdr--slim .brand-name {
        font-size: 15px;
    }
    .hdr--slim .hdr-right {
        display: none;
    }

    .notice {
        margin: 20px 40px 0;
        padding: 11px 14px;
        border: 1px solid var(--sr-line);
        border-left: 3px solid var(--sr-signal);
        border-radius: var(--sr-radius-control);
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        font-size: 13px;
    }

    // ── Title row ────────────────────────────────────────────────────────────
    .title-row {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 20px;
        padding: 34px 40px 20px;
    }
    .title {
        margin: 4px 0 0;
        font-size: 28px;
        font-weight: 700;
        letter-spacing: var(--sr-tighten);
        color: var(--sr-ink);
    }

    // ── Deck rows ────────────────────────────────────────────────────────────
    .rows {
        display: flex;
        flex-direction: column;
        gap: 14px;
        padding: 0 40px;
    }
    .row {
        display: grid;
        grid-template-columns: 1fr 300px auto;
        gap: 32px;
        align-items: center;
        padding: 22px 26px;
    }
    .row-info {
        min-width: 0;
    }
    .row-name {
        margin: 0;
        font-size: 18px;
        font-weight: 700;
        color: var(--sr-ink);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .row-meta {
        margin: 3px 0 0;
        font-family: var(--sr-mono);
        font-variant-numeric: tabular-nums;
        font-size: 11px;
        color: var(--sr-ink-3);
    }

    .row-progress-head {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        margin-bottom: 7px;
    }
    // Small mono labels the shared sheet doesn't cover (canvas COMPLETION / 62%).
    .prog-label {
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 10px;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .prog-value {
        font-family: var(--sr-mono);
        font-weight: 600;
        font-size: 12px;
        color: var(--sr-ink);
    }

    .row-actions {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .menu-wrap {
        position: relative;
    }
    // Delete lives here: a quiet overflow control, loud only on hover/focus.
    .more {
        display: grid;
        place-items: center;
        width: 34px;
        height: 34px;
        padding: 0;
        border: none;
        border-radius: var(--sr-radius-tile);
        background: none;
        color: var(--sr-faint);
        cursor: pointer;
        transition:
            background 0.12s ease,
            color 0.12s ease;
    }
    .more:hover,
    .more[aria-expanded="true"] {
        background: var(--sr-ghost);
        color: var(--sr-ink-slate);
    }

    .menu {
        position: absolute;
        top: calc(100% + 6px);
        right: 0;
        z-index: 50;
        width: max-content;
        padding: 14px;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-card-sm);
        box-shadow: var(--sr-shadow-card);
    }
    .menu-q {
        margin: 0 0 10px;
        font-size: 13px;
        color: var(--sr-ink);
    }
    .menu-actions {
        display: flex;
        gap: 8px;
    }

    .backdrop {
        position: fixed;
        inset: 0;
        z-index: 40;
        border: 0;
        padding: 0;
        background: transparent;
        cursor: default;
    }

    // ── New-deck affordance (dashed) ─────────────────────────────────────────
    .new-deck {
        display: flex;
        align-items: center;
        gap: 12px;
        width: 100%;
        padding: 18px 26px;
        border: 2px dashed var(--sr-line-dashed);
        border-radius: var(--sr-radius-card);
        background: transparent;
        color: var(--sr-ink-3);
        font-family: var(--sr-sans);
        text-align: left;
        cursor: pointer;
        transition: border-color 0.12s ease;
    }
    .new-deck:hover {
        border-color: var(--sr-signal);
    }
    .new-deck-plus {
        display: grid;
        place-items: center;
        width: 34px;
        height: 34px;
        border-radius: var(--sr-radius-control);
        background: var(--sr-track);
        font-size: 18px;
        font-weight: 600;
        color: var(--sr-ink-slate);
    }
    .new-deck-text {
        font-weight: 600;
        font-size: 14px;
        color: var(--sr-ink-slate);
    }

    // ── Empty state (2h) ─────────────────────────────────────────────────────
    .empty {
        padding: 64px 40px;
        text-align: center;
    }
    .empty-mark {
        display: grid;
        place-items: center;
        width: 64px;
        height: 64px;
        margin: 0 auto 18px;
        border: 2px dashed var(--sr-line-dashed);
        border-radius: var(--sr-radius-card);
        font-size: 26px;
        color: var(--sr-line-dashed);
    }
    .empty-title {
        margin: 0;
        font-size: 20px;
        font-weight: 700;
        color: var(--sr-ink);
    }
    .empty-help {
        max-width: 340px;
        margin: 7px auto 20px;
        font-size: 13px;
        line-height: 1.5;
        color: var(--sr-ink-3);
    }

    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    // ── Phone (2e): single column, chrome-free, >=44px targets ───────────────
    // The desktop brand/date/avatar header is OS-status-bar chrome on phone, so
    // it drops away and the title row leads (matches 2e's content).
    @media (max-width: $phone) {
        .wrap {
            padding-bottom: 28px;
        }
        .hdr {
            display: none;
        }
        .notice {
            margin: 16px 16px 0;
        }
        .title-row {
            padding: 20px 18px 14px;
        }
        .title {
            font-size: 22px;
        }
        .rows {
            padding: 0 14px;
            gap: 11px;
        }
        .row {
            display: flex;
            flex-direction: column;
            align-items: stretch;
            gap: 12px;
            padding: 15px;
        }
        .row-progress-head {
            margin-bottom: 6px;
        }
        .row-actions .sr-btn {
            flex: 1;
            min-height: 44px;
        }
        .more {
            width: 44px;
            height: 44px;
            flex: 0 0 auto;
        }
        .empty {
            padding: 56px 24px;
        }
        // Keep the empty-state CTA a comfortable touch target.
        .empty :global(.sr-btn) {
            min-height: 44px;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .new-deck,
        .more {
            transition: none;
        }
    }
</style>
