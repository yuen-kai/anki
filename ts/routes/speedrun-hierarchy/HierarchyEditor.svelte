<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto, replaceState } from "$app/navigation";
    import { onDestroy } from "svelte";
    import { get, writable } from "svelte/store";

    import ConceptModal from "./ConceptModal.svelte";
    import ConceptsPanel from "./ConceptsPanel.svelte";
    import HierarchyTree from "./HierarchyTree.svelte";
    import {
        findNode,
        findParent,
        hasUnsavedChanges,
        type Hierarchy,
        isLeaf,
        isUnsaved,
        type PulseState,
        saveHierarchy,
        type SaveStatus,
        saveStatusLabel,
        setTreeContext,
    } from "./lib";

    export let hierarchy: Hierarchy;
    // Where the back control goes. Defaults to the decks home; the backend-free
    // demo overrides it to step within its own gallery.
    export let onBack: () => void = () => goto("/speedrun-decks");
    // Whether edits can be saved to the backend. The demo runs against a private
    // clone with no collection, so it turns this off: edits still refresh the
    // views, but no Save control shows and no speedrun* RPC ever fires.
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
    // stays stable so a save always serialises the live deck id.
    let rev = 0;
    let destroyed = false;

    // Edits stay in the draft until an explicit save. A freshly loaded deck is
    // already on the backend, so it starts "saved"; a brand-new (unsaved) deck
    // stays silent until it is touched.
    let status: SaveStatus = persist && !isUnsaved(model.deckId) ? "saved" : "clean";
    let saveError = "";

    $: statusLabel = saveStatusLabel(status);
    $: canSave = persist && (status === "dirty" || status === "error");

    // An edit only marks the draft dirty; nothing is written until Save. The
    // demo (persist=false) still refreshes the views through `rev`, with no
    // status and no RPC.
    function change(): void {
        rev += 1;
        if (persist) {
            status = "dirty";
            saveError = "";
        }
    }

    async function save(): Promise<void> {
        if (!persist || status === "saving") {
            return;
        }
        status = "saving";
        saveError = "";
        try {
            const result = await saveHierarchy(model);
            if (destroyed) {
                return;
            }
            if (result.deckId && result.deckId !== model.deckId) {
                model.deckId = result.deckId;
                // Adopt the minted id in the URL without reloading, so the draft
                // and current selection survive the create.
                replaceState(`/speedrun-hierarchy/${result.deckId}`, {});
            }
            // An edit landing mid-save leaves the draft dirty; an empty id means
            // the backend persisted nothing (e.g. no deck name), so stay unsaved.
            if (status === "saving") {
                status = result.deckId || !isUnsaved(model.deckId) ? "saved" : "dirty";
            }
        } catch (error) {
            if (destroyed) {
                return;
            }
            saveError = error instanceof Error ? error.message : String(error);
            status = "error";
        }
    }

    function pulse(chain: string[]): void {
        pulseSeq += 1;
        pulseState.set({ ids: chain, seq: pulseSeq });
    }

    // Publishing the same selection would still notify every subscriber and
    // re-run the tree's reactive work, so only set it when it actually changes.
    function select(id: string | null): void {
        if (get(selectedId) !== id) {
            selectedId.set(id);
        }
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

    // Leaving with unsaved edits should warn first: the back control confirms,
    // and a hard unload (refresh or close) triggers the native prompt.
    function requestBack(): void {
        if (
            persist &&
            hasUnsavedChanges(status) &&
            !confirm("You have unsaved changes. Leave without saving?")
        ) {
            return;
        }
        onBack();
    }

    function onBeforeUnload(event: BeforeUnloadEvent): void {
        if (persist && hasUnsavedChanges(status)) {
            event.preventDefault();
            // Older browsers still need returnValue set to show the prompt.
            event.returnValue = "";
        }
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
    });
</script>

<svelte:window on:beforeunload={onBeforeUnload} />

<div class="editor">
    <div class="inner">
        <div class="grid">
            <section class="col-full">
                <div class="card structure">
                    <header class="deckbar">
                        <div class="lead">
                            <button type="button" class="back" on:click={requestBack}>
                                ← Decks
                            </button>
                            <input
                                class="deck-title"
                                bind:value={model.root.title}
                                on:input={onDeckInput}
                                placeholder="Deck name"
                                aria-label="Deck name"
                            />
                        </div>
                        {#if persist}
                            <div class="save">
                                <span
                                    class="status"
                                    class:status--error={status === "error"}
                                    aria-live="polite"
                                    title={status === "error" ? saveError : undefined}
                                >
                                    {statusLabel}
                                </span>
                                <button
                                    type="button"
                                    class="save-btn"
                                    on:click={save}
                                    disabled={!canSave}
                                >
                                    {status === "saving" ? "Saving" : "Save"}
                                </button>
                            </div>
                        {/if}
                    </header>

                    <HierarchyTree root={model.root} {rev} />
                </div>
            </section>

            <section class="col">
                <div class="card tall" class:empty={!selectedLeaf}>
                    {#if selectedLeaf}
                        <ConceptsPanel
                            node={selectedLeaf}
                            {rev}
                            {parentName}
                            {selectedConceptId}
                            onSelect={selectConcept}
                        />
                    {:else}
                        <p class="placeholder">Open a topic to see its concepts</p>
                    {/if}
                </div>
            </section>

            <section class="col">
                <div class="card tall" class:empty={!activeConcept}>
                    {#if activeConcept}
                        <ConceptModal concept={activeConcept} onChange={change} />
                    {:else}
                        <p class="placeholder">Open a concept to edit it</p>
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
        min-height: 100vh;
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
    // minmax(0, 1fr) (not the default 1fr = minmax(auto, 1fr)) lets a column
    // shrink below its content's min width, so long topic names ellipsize
    // inside the card instead of forcing the whole grid wider than a phone.
    .grid {
        display: grid;
        grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
        gap: 22px;
    }
    .col-full {
        grid-column: 1 / -1;
        min-width: 0;
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
    // Before a topic/concept is open the panel has nothing to show; keep it
    // compact and centre the hint so the space reads intentional, not broken.
    .tall.empty {
        min-height: 200px;
        display: grid;
        place-items: center;
    }
    .tall.empty .placeholder {
        text-align: center;
    }

    .deckbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        margin-bottom: 18px;
    }
    .lead {
        flex: 1 1 auto;
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

        // Fill the space between the back control and the save cluster so a
        // full deck name shows instead of truncating at a fixed cap.
        flex: 1 1 auto;
        min-width: 0;
    }
    .deck-title::placeholder {
        color: var(--sr-faint);
        font-weight: 600;
    }

    // The save cluster: a quiet mono readout beside the explicit Save control.
    .save {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        gap: 12px;
    }
    .status {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        white-space: nowrap;
    }
    .status--error {
        color: var(--sr-signal-deep);
    }
    .save-btn {
        @include syn.btn;
        @include syn.btn-primary;

        padding: 8px 18px;
        font-size: 12px;
    }
    .save-btn:hover:not(:disabled) {
        filter: brightness(1.04);
    }
    .save-btn:disabled {
        @include syn.btn-disabled;
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
            grid-template-columns: minmax(0, 1fr);
        }
    }
    @media (max-width: 34rem) {
        .editor {
            padding: 1.25rem 1rem 2.5rem;
        }
        .card {
            padding: 20px 18px;
        }
        .tall,
        .tall.empty {
            min-height: 0;
        }
        // A large 22px title overruns the narrow header and clips; 18px fits a
        // full deck name on the phone row.
        .deck-title {
            font-size: 18px;
        }
        // Let the header stack rather than crush the title, and give the touch
        // controls a full 44px target.
        .deckbar {
            flex-wrap: wrap;
        }
        .back {
            display: inline-flex;
            align-items: center;
            min-height: 44px;
        }
        .save-btn {
            min-height: 44px;
        }
    }
</style>
