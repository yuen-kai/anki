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
    import { tick } from "svelte";

    import type { DeckRow } from "./lib";

    export let decks: DeckRow[];
    export let loadError: string | null = null;
    export let onStudy: (deck: DeckRow) => Promise<void> | void;
    export let onDetails: (deck: DeckRow) => void;
    export let onCreate: () => void;
    export let onDelete: (deck: DeckRow) => Promise<void> | void;
    // onAccount powers the profile avatar (account + sync): the header avatar on
    // desktop, and a phone-appropriate avatar in the mobile app-nav where that
    // header is hidden. The mobile shell also surfaces Demo there. Undefined
    // handlers (the demo route) render those as inert.
    export let mobile = false;
    export let onAccount: (() => void) | undefined = undefined;
    export let onDemo: (() => void) | undefined = undefined;

    let rows: DeckRow[] = decks;
    let busy = false;
    // The deck whose row is showing its inline delete confirm, if any.
    let confirmingId: string | null = null;
    // Per-row trash buttons, so focus returns to one after its confirm closes.
    const trashButtons: Record<string, HTMLButtonElement | null> = {};

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
            confirmingId = null;
        } catch (err) {
            loadError = message(err);
        } finally {
            busy = false;
        }
    }

    function askDelete(deck: DeckRow): void {
        confirmingId = deck.deckId;
    }

    // Close the inline confirm and hand focus back to the row's trash control.
    async function cancelDelete(): Promise<void> {
        const id = confirmingId;
        confirmingId = null;
        await tick();
        if (id) {
            trashButtons[id]?.focus();
        }
    }

    // Land focus on the confirm's safe (Cancel) control when it opens.
    function focusOnShow(node: HTMLElement): void {
        node.focus();
    }

    function onWindowKeydown(event: KeyboardEvent): void {
        if (event.key === "Escape" && confirmingId) {
            void cancelDelete();
        }
    }

    const plural = (n: number, word: string): string =>
        `${n} ${word}${n === 1 ? "" : "s"}`;
    const percent = (fraction: number): number => Math.round(fraction * 100);
</script>

<svelte:window on:keydown={onWindowKeydown} />

<div class="screen">
    <div class="wrap">
        {#if mobile && (onAccount || onDemo)}
            <nav class="app-nav" aria-label="Speedrun">
                {#if onDemo}
                    <button type="button" class="app-nav-link" on:click={onDemo}>
                        Demo
                    </button>
                {/if}
                {#if onAccount}
                    <button
                        type="button"
                        class="app-nav-avatar"
                        aria-label="Account"
                        on:click={onAccount}
                    >
                        <svg
                            viewBox="0 0 24 24"
                            width="20"
                            height="20"
                            fill="currentColor"
                        >
                            <circle cx="12" cy="9" r="3.4" />
                            <path d="M5.5 19.5c.5-3.5 3.2-5.4 6.5-5.4s6 1.9 6.5 5.4z" />
                        </svg>
                    </button>
                {/if}
            </nav>
        {/if}
        <header class="hdr" class:hdr--slim={isEmpty}>
            <div class="brand">
                <span class="sr-logo" aria-hidden="true"></span>
                <span class="brand-name">Synapse</span>
                <span class="brand-tag">MCAT</span>
            </div>
            <div class="hdr-right">
                <span class="hdr-date">{today}</span>
                {#if onAccount}
                    <button
                        type="button"
                        class="avatar"
                        aria-label="Account"
                        on:click={onAccount}
                    >
                        <svg
                            viewBox="0 0 24 24"
                            width="18"
                            height="18"
                            fill="currentColor"
                        >
                            <circle cx="12" cy="9" r="3.4" />
                            <path d="M5.5 19.5c.5-3.5 3.2-5.4 6.5-5.4s6 1.9 6.5 5.4z" />
                        </svg>
                    </button>
                {:else}
                    <span class="avatar" aria-hidden="true">
                        <svg
                            viewBox="0 0 24 24"
                            width="18"
                            height="18"
                            fill="currentColor"
                        >
                            <circle cx="12" cy="9" r="3.4" />
                            <path d="M5.5 19.5c.5-3.5 3.2-5.4 6.5-5.4s6 1.9 6.5 5.4z" />
                        </svg>
                    </span>
                {/if}
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
            <div class="list">
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
                                {#if confirmingId === deck.deckId}
                                    <div
                                        class="confirm"
                                        role="group"
                                        aria-label="Delete deck"
                                    >
                                        <span class="confirm-q">Delete deck?</span>
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
                                            on:click={cancelDelete}
                                            disabled={busy}
                                            use:focusOnShow
                                        >
                                            Cancel
                                        </button>
                                    </div>
                                {:else}
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
                                    <button
                                        type="button"
                                        class="trash"
                                        aria-label="Delete deck"
                                        on:click={() => askDelete(deck)}
                                        bind:this={trashButtons[deck.deckId]}
                                        disabled={busy}
                                    >
                                        <svg
                                            viewBox="0 0 24 24"
                                            width="18"
                                            height="18"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="1.7"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            aria-hidden="true"
                                        >
                                            <path d="M4 7h16" />
                                            <path
                                                d="M9 7V5.5A1.5 1.5 0 0 1 10.5 4h3A1.5 1.5 0 0 1 15 5.5V7"
                                            />
                                            <path
                                                d="M6.5 7l.8 12.1A2 2 0 0 0 9.3 21h5.4a2 2 0 0 0 2-1.9L17.5 7"
                                            />
                                            <path d="M10 11v6M14 11v6" />
                                        </svg>
                                    </button>
                                {/if}
                            </div>
                        </article>
                    {/each}

                    <button type="button" class="new-deck" on:click={onCreate}>
                        <span class="new-deck-plus" aria-hidden="true">+</span>
                        <span class="new-deck-text">New deck</span>
                    </button>
                </div>
            </div>
        {/if}
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    $phone: 40rem;

    .screen {
        box-sizing: border-box;
        min-height: 100vh;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.5;
        -webkit-font-smoothing: antialiased;
        @include syn.host; // tokens + .sr-* classes; keep LAST
    }

    // Fill the viewport so the empty state can center in the space below the
    // header. The populated list flows from the top: a short list leaves natural
    // room beneath it rather than floating in the middle of the page.
    .wrap {
        box-sizing: border-box;
        min-height: 100vh;
        display: flex;
        flex-direction: column;
        max-width: 1240px;
        margin: 0 auto;
        padding-bottom: 40px;
    }

    // Mobile-shell top nav: the account avatar and the Demo entry (the desktop
    // keeps these in the header and Tools menu). A slim, quiet bar above the
    // decks, out of the way.
    .app-nav {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 8px;
        padding: 8px 14px;
        border-bottom: 1px solid var(--sr-line-2);
    }
    .app-nav-link {
        padding: 8px 12px;
        min-height: 40px;
        border: none;
        border-radius: var(--sr-radius-control);
        background: none;
        color: var(--sr-ink-2);
        font-family: var(--sr-sans);
        font-size: 13px;
        font-weight: 540;
        cursor: pointer;
    }
    .app-nav-link:hover {
        background: var(--sr-ghost);
        color: var(--sr-ink);
    }
    // Phone profile entry: shown only at phone width, where the desktop header
    // (and its avatar) is hidden, so account stays reachable. >=44px touch.
    .app-nav-avatar {
        display: none;
        place-items: center;
        width: 44px;
        height: 44px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: var(--sr-track);
        color: var(--sr-ink-slate);
        cursor: pointer;
    }
    .app-nav-avatar:hover {
        color: var(--sr-ink);
    }
    // Keep the circle on focus (the shared :focus-visible rule squares corners).
    .app-nav-avatar:focus-visible {
        border-radius: 50%;
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
    // Profile entry point to account + sync: a button when onAccount is set,
    // else a decorative span on the demo route. Neutral person glyph since
    // there's no user identity to key initials off.
    .avatar {
        display: grid;
        place-items: center;
        width: 32px;
        height: 32px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: var(--sr-track);
        color: var(--sr-ink-slate);
    }
    button.avatar {
        cursor: pointer;
        transition: color 0.12s ease;
    }
    button.avatar:hover {
        color: var(--sr-ink);
    }
    // Keep the circle on focus (the shared :focus-visible rule squares corners).
    button.avatar:focus-visible {
        border-radius: 50%;
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

    // A single, direct delete control per row: quiet by default, coral on
    // hover/focus so its intent reads before it is pressed.
    .trash {
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
    .trash:hover,
    .trash:focus-visible {
        background: var(--sr-signal-weak);
        color: var(--sr-signal);
    }
    .trash:disabled {
        background: none;
        color: var(--sr-faint);
        cursor: not-allowed;
    }

    // The inline confirm that swaps into the row action area in place, with no
    // overlay or popover (so opening it never shifts the layout or flashes).
    .confirm {
        display: flex;
        align-items: center;
        gap: 10px;
    }
    .confirm-q {
        font-size: 13px;
        font-weight: 600;
        color: var(--sr-ink);
        white-space: nowrap;
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
        margin-block: auto;
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
        // Header (with its avatar) is gone on phone, so surface the avatar here.
        .app-nav-avatar {
            display: grid;
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
        .trash {
            width: 44px;
            height: 44px;
            flex: 0 0 auto;
        }
        // Let the confirm fill the row: question on its own line, buttons below.
        .confirm {
            flex: 1;
            flex-wrap: wrap;
        }
        .confirm-q {
            flex-basis: 100%;
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
        .trash,
        button.avatar {
            transition: none;
        }
    }
</style>
