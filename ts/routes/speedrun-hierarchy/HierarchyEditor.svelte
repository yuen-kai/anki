<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto, replaceState } from "$app/navigation";
    import { onDestroy } from "svelte";
    import { writable } from "svelte/store";

    import ConceptsPanel from "./ConceptsPanel.svelte";
    import HierarchyTree from "./HierarchyTree.svelte";
    import {
        createAutosave,
        findNode,
        type Hierarchy,
        isLeaf,
        isUnsaved,
        type PulseState,
        type SaveResult,
        setTreeContext,
    } from "./lib";

    export let hierarchy: Hierarchy;

    // The loaded blob is a fresh parse each visit, so we edit it in place as the
    // single source of truth and let the shared graph propagate changes.
    const model: Hierarchy = hierarchy;

    const selectedId = writable<string | null>(null);
    const pulseState = writable<PulseState | null>(null);
    let pulseSeq = 0;

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

    $: selectedNode = keep(rev, findNode(model.root, $selectedId));
    $: selectedLeaf = selectedNode && isLeaf(selectedNode) ? selectedNode : null;
    $: deckName = keep(rev, model.root.title.trim());
    $: draft = isUnsaved(model.deckId);
    // R5: the create flow must read "Create Hierarchy"; the edit flow mirrors it.
    $: pageTitle = draft ? "Create Hierarchy" : "Edit Hierarchy";

    onDestroy(() => {
        destroyed = true;
        autosave.flush();
        autosave.cancel();
    });
</script>

<div class="editor">
    <div class="inner">
        <header class="bar">
            <div class="lead">
                <nav class="crumbs" aria-label="Breadcrumb">
                    <button class="back" on:click={() => goto("/speedrun-decks")}>
                        Decks
                    </button>
                    <span class="sep" aria-hidden="true">/</span>
                    <span class="here">{deckName || "Untitled deck"}</span>
                </nav>
                <h1 class="page-title">{pageTitle}</h1>
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

        <div class="workspace">
            <section class="pane tree-pane" aria-label="Deck structure">
                <p class="pane-label">Structure</p>
                <HierarchyTree root={model.root} />
            </section>

            <section class="pane detail-pane" aria-label="Concepts">
                {#if selectedLeaf}
                    <ConceptsPanel node={selectedLeaf} />
                {:else}
                    <div class="placeholder">
                        <p>Select a leaf topic to add concepts.</p>
                    </div>
                {/if}
            </section>
        </div>
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    .editor {
        @include sr.tokens;

        box-sizing: border-box;
        min-height: 100%;
        padding: 1.75rem 1.75rem 3rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;
    }

    .inner {
        max-width: 66rem;
        margin: 0 auto;
    }

    .bar {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 1rem;
        flex-wrap: wrap;
        padding-bottom: 0.9rem;
        border-bottom: 2px solid var(--sr-ink);
    }
    .lead {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
        min-width: 0;
    }
    .crumbs {
        display: flex;
        align-items: center;
        gap: 0.55rem;
        min-width: 0;
        font-size: 0.82rem;
    }
    .page-title {
        margin: 0;
        font-size: 1.7rem;
        font-weight: 720;
        letter-spacing: -0.02em;
        line-height: 1.1;
    }
    .back {
        border: none;
        background: none;
        padding: 0;
        font: inherit;
        font-weight: 560;
        color: var(--sr-ink-2);
        cursor: pointer;
    }
    .back:hover {
        color: var(--sr-ink);
    }
    .sep {
        color: var(--sr-ink-3);
    }
    .here {
        font-weight: 560;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    // A quiet mono instrument readout, aligned with the breadcrumb eyebrow.
    .status {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        white-space: nowrap;
        padding-top: 0.15rem;
    }

    .workspace {
        display: grid;
        grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
        gap: 1.25rem;
        margin-top: 1.5rem;
        align-items: start;
    }
    .pane {
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        padding: 1rem 1.1rem 1.2rem;
    }
    .pane-label {
        margin: 0 0 0.6rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .detail-pane {
        position: sticky;
        top: 1rem;
    }
    .placeholder {
        color: var(--sr-ink-3);
        font-size: 0.9rem;
        padding: 1.5rem 0.25rem;
    }
    .placeholder p {
        margin: 0;
    }

    :global(.editor :focus-visible) {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    @media (max-width: 52rem) {
        .workspace {
            grid-template-columns: 1fr;
        }
        .detail-pane {
            position: static;
        }
    }
    @media (max-width: 34rem) {
        .editor {
            padding: 1.25rem 1rem 2.5rem;
        }
    }
</style>
