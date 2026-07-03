<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto, replaceState } from "$app/navigation";
    import { onDestroy } from "svelte";
    import { writable } from "svelte/store";

    import ConceptModal from "./ConceptModal.svelte";
    import ConceptsPanel from "./ConceptsPanel.svelte";
    import HierarchyTree from "./HierarchyTree.svelte";
    import {
        createAutosave,
        findNode,
        findParent,
        type Hierarchy,
        isLeaf,
        type PulseState,
        type SaveResult,
        setTreeContext,
    } from "./lib";

    export let hierarchy: Hierarchy;
    // Where the back control goes. Defaults to the decks home; the backend-free
    // demo overrides it to step within its own gallery.
    export let onBack: () => void = () => goto("/speedrun-decks");
    // Whether edits autosave to the backend. The demo runs against a private
    // clone with no collection, so it turns this off: edits still refresh the
    // views, but no speedrun* RPC ever fires.
    export let persist = true;

    // The loaded blob is a fresh parse each visit, so we edit it in place as the
    // single source of truth and let the shared graph propagate changes.
    const model: Hierarchy = hierarchy;

    const selectedId = writable<string | null>(null);
    const pulseState = writable<PulseState | null>(null);
    let pulseSeq = 0;

    // Which concept (inside the open leaf) is loaded in the editor pane.
    let selectedConceptId: string | null = null;

    // Child components mutate the shared tree in place, which does not notify
    // this component; bumping `rev` on every edit is the manual dependency that
    // forces the derived values below to recompute. The model object identity
    // stays stable so the autosave always serialises the live deck id.
    let rev = 0;
    let destroyed = false;

    type Status = "idle" | "saving" | "saved" | "error";
    let status: Status = "idle";
    let saveError = "";

    const autosave = createAutosave(onSaved, onSaveError);

    function onSaved(result: SaveResult): void {
        if (destroyed) {
            return;
        }
        // An empty id means the save was a no-op (empty deck name); leave the
        // draft as unsaved until there is something to persist.
        if (result.deckId && result.deckId !== model.deckId) {
            model.deckId = result.deckId;
            // Adopt the minted id in the URL without reloading, so the in-memory
            // draft (and current selection) survives the create.
            replaceState(`/speedrun-hierarchy/${result.deckId}`, {});
        }
        status = "saved";
    }

    function onSaveError(error: unknown): void {
        if (destroyed) {
            return;
        }
        status = "error";
        saveError = error instanceof Error ? error.message : String(error);
    }

    function change(): void {
        rev += 1;
        if (!persist) {
            return;
        }
        status = "saving";
        autosave.schedule(model);
    }

    function pulse(chain: string[]): void {
        pulseSeq += 1;
        pulseState.set({ ids: chain, seq: pulseSeq });
    }

    function select(id: string | null): void {
        selectedId.set(id);
    }

    setTreeContext({ change, pulse, select, selectedId, pulseState });

    // Passing `rev` in makes it a tracked dependency without changing the value.
    function keep<T>(_rev: number, value: T): T {
        return value;
    }

    function selectConcept(id: string | null): void {
        selectedConceptId = id;
    }

    function onDeckInput(): void {
        change();
    }
    function onDeckCommit(): void {
        change();
        pulse([model.root.id]);
    }

    $: selectedNode = keep(rev, findNode(model.root, $selectedId));
    $: selectedLeaf = selectedNode && isLeaf(selectedNode) ? selectedNode : null;
    $: parentName = selectedLeaf
        ? findParent(model.root, selectedLeaf.id)?.title.trim() || null
        : null;

    // Drop the loaded concept whenever the open leaf changes, so the editor pane
    // never shows a concept from a topic that is no longer selected.
    let openLeafId: string | null = null;
    $: {
        const id = selectedLeaf ? selectedLeaf.id : null;
        if (id !== openLeafId) {
            openLeafId = id;
            selectedConceptId = null;
        }
    }

    $: activeConcept = keep(
        rev,
        selectedLeaf && selectedConceptId
            ? (selectedLeaf.concepts.find((c) => c.id === selectedConceptId) ?? null)
            : null,
    );

    onDestroy(() => {
        destroyed = true;
        if (persist) {
            autosave.flush();
        }
        autosave.cancel();
    });
</script>

<div class="editor">
    <div class="inner">
        <div class="grid">
            <section class="col-full">
                <div class="card structure">
                    <header class="deckbar">
                        <div class="lead">
                            <button type="button" class="back" on:click={onBack}>
                                ← Decks
                            </button>
                            <input
                                class="deck-title"
                                bind:value={model.root.title}
                                on:input={onDeckInput}
                                on:change={onDeckCommit}
                                placeholder="Deck name"
                                aria-label="Deck name"
                            />
                        </div>
                        <div class="status" aria-live="polite">
                            {#if status === "saving"}
                                Saving
                            {:else if status === "saved"}
                                Saved
                            {:else if status === "error"}
                                <span title={saveError}>Save failed</span>
                            {/if}
                        </div>
                    </header>

                    <HierarchyTree root={model.root} {rev} />
                </div>
            </section>

            <section class="col">
                <div class="card tall">
                    {#if selectedLeaf}
                        <ConceptsPanel
                            node={selectedLeaf}
                            {rev}
                            {parentName}
                            {selectedConceptId}
                            onSelect={selectConcept}
                        />
                    {:else}
                        <p class="placeholder">No topic open</p>
                    {/if}
                </div>
            </section>

            <section class="col">
                <div class="card tall">
                    {#if activeConcept}
                        <ConceptModal concept={activeConcept} onChange={change} />
                    {:else}
                        <p class="placeholder">No concept open</p>
                    {/if}
                </div>
            </section>
        </div>
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .editor {
        box-sizing: border-box;
        min-height: 100%;
        padding: 1.75rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;

        @include syn.sr-tokens;
    }

    .inner {
        max-width: 74rem;
        margin: 0 auto;
    }

    // Deck structure spans both columns; the concepts list and concept editor
    // sit side by side beneath it, matching the builder frame.
    .grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 22px;
    }
    .col-full {
        grid-column: 1 / -1;
    }
    .col {
        min-width: 0;
    }

    .card {
        @include syn.card;

        padding: 24px 26px;
    }
    .tall {
        padding: 22px 24px;
        min-height: 420px;
    }

    .deckbar {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
        gap: 1rem;
        margin-bottom: 18px;
    }
    .lead {
        display: flex;
        align-items: baseline;
        gap: 12px;
        min-width: 0;
    }
    .back {
        flex-shrink: 0;
        border: none;
        background: none;
        padding: 0;
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 12px;
        color: var(--sr-ink-3);
        cursor: pointer;
    }
    .back:hover {
        color: var(--sr-ink);
    }
    .deck-title {
        @include syn.input-underline;

        flex: 1 1 13rem;
        min-width: 0;
        max-width: 16rem;
    }
    .deck-title::placeholder {
        color: var(--sr-faint);
        font-weight: 600;
    }

    // A quiet mono instrument readout in the header's spare slot.
    .status {
        flex-shrink: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        white-space: nowrap;
    }

    .placeholder {
        margin: 0;
        color: var(--sr-faint);
        font-size: 0.9rem;
    }

    :global(.editor :focus-visible) {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    @media (max-width: 60rem) {
        .grid {
            grid-template-columns: 1fr;
        }
    }
    @media (max-width: 34rem) {
        .editor {
            padding: 1.25rem 1rem 2.5rem;
        }
        .card {
            padding: 20px 18px;
        }
        .tall {
            min-height: 0;
        }
    }
</style>
