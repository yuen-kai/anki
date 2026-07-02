<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto } from "$app/navigation";

    import {
        type DeckSummary,
        deleteDeck,
        listDecks,
        openDeck,
    } from "../speedrun-hierarchy/lib";
    import type { PageData } from "./$types";

    export let data: PageData;

    let rows: DeckSummary[] = data.decks;
    let loadError: string | null = data.error;
    let busy = false;
    let confirmingId: string | null = null;

    async function refresh(): Promise<void> {
        try {
            rows = await listDecks();
            loadError = null;
        } catch (err) {
            loadError = err instanceof Error ? err.message : String(err);
        }
    }

    async function openOverview(deck: DeckSummary): Promise<void> {
        if (busy) {
            return;
        }
        busy = true;
        try {
            await openDeck(deck.deckId);
        } catch (err) {
            loadError = err instanceof Error ? err.message : String(err);
        } finally {
            busy = false;
        }
    }

    function create(): void {
        goto("/speedrun-hierarchy/new");
    }

    function edit(deck: DeckSummary): void {
        goto(`/speedrun-hierarchy/${deck.deckId}`);
    }

    async function confirmDelete(deck: DeckSummary): Promise<void> {
        busy = true;
        try {
            await deleteDeck(deck.deckId);
            confirmingId = null;
            await refresh();
        } catch (err) {
            loadError = err instanceof Error ? err.message : String(err);
        } finally {
            busy = false;
        }
    }
</script>

<div class="decks">
    <div class="inner">
        <header class="head">
            <h1 class="title">Decks</h1>
            <button class="btn primary" on:click={create}>Create deck</button>
        </header>

        {#if loadError}
            <p class="notice" role="alert">Could not load decks: {loadError}</p>
        {/if}

        {#if rows.length === 0}
            <div class="empty">
                <p class="empty-title">No decks yet</p>
                <p class="empty-line">Create a deck to start a topic hierarchy.</p>
                <button class="btn primary" on:click={create}>Create deck</button>
            </div>
        {:else}
            <table class="grid">
                <thead>
                    <tr>
                        <th scope="col" class="c-name">Name</th>
                        <th scope="col" class="c-todo">To do</th>
                        <th scope="col" class="c-act">
                            <span class="sr-only">Actions</span>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {#each rows as deck (deck.deckId)}
                        <tr class:confirming={confirmingId === deck.deckId}>
                            <td class="c-name">
                                <button
                                    class="name"
                                    on:click={() => openOverview(deck)}
                                    disabled={busy}
                                >
                                    {deck.name || "Untitled deck"}
                                </button>
                            </td>
                            <td class="c-todo">
                                <span class="todo" class:zero={deck.todo === 0}>
                                    {deck.todo}
                                </span>
                            </td>
                            <td class="c-act">
                                {#if confirmingId === deck.deckId}
                                    <span class="confirm">
                                        <span class="confirm-q">Delete deck?</span>
                                        <button
                                            class="btn solid"
                                            on:click={() => confirmDelete(deck)}
                                            disabled={busy}
                                        >
                                            Delete
                                        </button>
                                        <button
                                            class="btn ghost"
                                            on:click={() => (confirmingId = null)}
                                            disabled={busy}
                                        >
                                            Cancel
                                        </button>
                                    </span>
                                {:else}
                                    <button
                                        class="btn ghost"
                                        on:click={() => edit(deck)}
                                        disabled={busy}
                                    >
                                        Edit
                                    </button>
                                    <button
                                        class="btn ghost"
                                        on:click={() => (confirmingId = deck.deckId)}
                                        disabled={busy}
                                    >
                                        Delete
                                    </button>
                                {/if}
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        {/if}
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    .decks {
        @include sr.tokens;

        box-sizing: border-box;
        min-height: 100%;
        padding: 2.25rem 1.75rem 3rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;
    }

    .inner {
        max-width: 52rem;
        margin: 0 auto;
    }

    .head {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        padding-bottom: 1rem;
        border-bottom: 2px solid var(--sr-ink);
    }
    .title {
        margin: 0;
        font-size: 1.7rem;
        font-weight: 720;
        letter-spacing: -0.02em;
    }

    .notice {
        margin: 1.25rem 0 0;
        padding: 0.7rem 0.9rem;
        border: 1px solid var(--sr-line-2);
        border-left: 3px solid var(--sr-ink-3);
        border-radius: 6px;
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        font-size: 0.9rem;
    }

    // A precise ledger: sits on the paper, ruled rows, mono tabular counts.
    .grid {
        width: 100%;
        margin-top: 1.5rem;
        border-collapse: collapse;
    }
    th {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        text-align: left;
        padding: 0 0.75rem 0.6rem;
        border-bottom: 1.5px solid var(--sr-ink);
    }
    .c-todo {
        width: 7rem;
        text-align: right;
    }
    .c-act {
        width: 1%;
        white-space: nowrap;
        text-align: right;
        opacity: 0;
        transition: opacity 0.12s ease;
    }
    td {
        padding: 0 0.75rem;
        border-bottom: 1px solid var(--sr-line);
        vertical-align: middle;
    }
    tbody tr:hover {
        background: var(--sr-panel-2);
    }
    // Actions stay hidden until the row is hovered, focused, or confirming.
    tbody tr:hover .c-act,
    tbody tr:focus-within .c-act,
    tbody tr.confirming .c-act {
        opacity: 1;
    }

    .name {
        display: block;
        width: 100%;
        padding: 0.7rem 0;
        border: none;
        background: none;
        color: var(--sr-ink);
        font: inherit;
        font-size: 0.98rem;
        font-weight: 560;
        text-align: left;
        cursor: pointer;
    }
    .name:hover {
        text-decoration: underline;
        text-underline-offset: 3px;
    }
    .name:disabled {
        cursor: default;
    }

    td.c-todo {
        text-align: right;
    }
    .todo {
        font-family: var(--sr-mono);
        font-variant-numeric: tabular-nums;
        font-size: 0.9rem;
        color: var(--sr-ink);
    }
    .todo.zero {
        color: var(--sr-ink-3);
    }

    .confirm {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
    }
    .confirm-q {
        font-size: 0.85rem;
        color: var(--sr-ink-2);
    }

    .btn {
        appearance: none;
        border: 1px solid transparent;
        border-radius: 6px;
        padding: 0.45rem 0.85rem;
        font-family: var(--sr-sans);
        font-size: 0.85rem;
        font-weight: 560;
        line-height: 1;
        cursor: pointer;
        transition:
            background 0.12s ease,
            color 0.12s ease;
    }
    .btn:disabled {
        opacity: 0.5;
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
    .btn.ghost {
        background: none;
        color: var(--sr-ink-2);
        border-color: var(--sr-line-2);
        margin-left: 0.4rem;
    }
    .btn.ghost:first-child {
        margin-left: 0;
    }
    .btn.ghost:hover:not(:disabled) {
        background: var(--sr-panel-2);
        color: var(--sr-ink);
    }
    // Destructive confirm: a solid ink button, no second accent colour.
    .btn.solid {
        background: var(--sr-ink);
        color: var(--sr-paper);
        border-color: var(--sr-ink);
    }
    .btn.solid:hover:not(:disabled) {
        filter: brightness(1.1);
    }

    .empty {
        margin-top: 3rem;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.4rem;
    }
    .empty-title {
        margin: 0;
        font-size: 1.05rem;
        font-weight: 640;
    }
    .empty-line {
        margin: 0 0 0.7rem;
        color: var(--sr-ink-2);
    }

    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
    }

    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    @media (max-width: 34rem) {
        .decks {
            padding: 1.5rem 1rem 2.5rem;
        }
        .head {
            flex-direction: column;
            align-items: stretch;
        }
        .head .btn.primary {
            width: 100%;
        }
        th,
        td {
            padding-left: 0.6rem;
            padding-right: 0.6rem;
        }
    }
</style>
