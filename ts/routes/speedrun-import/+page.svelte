<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto } from "$app/navigation";

    import type { Hierarchy } from "../speedrun-hierarchy/lib";
    import Chat from "./Chat.svelte";
    import DeckPreview from "./DeckPreview.svelte";
    import Intake from "./Intake.svelte";
    import {
        aiImport,
        type ChatMsg,
        type DeckCounts,
        extractFile,
        type ExtractedFile,
        type ImportResponse,
        openingMessage,
        openInBuilder,
        sourcesForRequest,
    } from "./lib";
    import type { PageData } from "./$types";

    const { data }: { data: PageData } = $props();

    const hostAvailable = $derived(data.hostAvailable);
    const available = $derived(data.available);

    type Phase = "intake" | "chat" | "preview";
    let phase = $state<Phase>("intake");

    let files = $state<ExtractedFile[]>([]);
    let note = $state("");
    let extracting = $state(false);

    let messages = $state<ChatMsg[]>([]);
    let draft = $state("");
    let ready = $state(false);

    let deck = $state<{ hierarchy: Hierarchy; summary: DeckCounts } | null>(null);

    let busy = $state(false);
    let error = $state("");

    function describe(err: unknown): string {
        return err instanceof Error ? err.message : String(err);
    }

    async function addFiles(added: File[]): Promise<void> {
        extracting = true;
        try {
            for (const file of added) {
                const extracted = await extractFile(file);
                files = [...files, extracted];
            }
        } finally {
            extracting = false;
        }
    }

    function removeFile(index: number): void {
        files = files.filter((_, i) => i !== index);
    }

    function handleResponse(response: ImportResponse): void {
        if (response.kind === "error") {
            error = response.message;
        } else if (response.kind === "message") {
            messages = [...messages, { role: "assistant", content: response.content }];
            ready = response.ready;
        } else {
            deck = { hierarchy: response.hierarchy, summary: response.summary };
            phase = "preview";
        }
    }

    async function runTurn(turnPhase: "clarify" | "generate"): Promise<void> {
        busy = true;
        error = "";
        try {
            handleResponse(
                await aiImport({
                    sources: sourcesForRequest(files),
                    messages,
                    phase: turnPhase,
                }),
            );
        } catch (err) {
            error = describe(err);
        } finally {
            busy = false;
        }
    }

    function toChat(): void {
        if (messages.length === 0) {
            messages = [{ role: "user", content: openingMessage(note) }];
            phase = "chat";
            void runTurn("clarify");
        } else {
            phase = "chat";
        }
    }

    function send(): void {
        const text = draft.trim();
        if (!text) {
            return;
        }
        messages = [...messages, { role: "user", content: text }];
        draft = "";
        ready = false;
        void runTurn("clarify");
    }

    async function openBuilder(): Promise<void> {
        if (!deck) {
            return;
        }
        busy = true;
        error = "";
        try {
            await openInBuilder(deck.hierarchy);
        } catch (err) {
            error = describe(err);
            busy = false;
        }
    }

    function discard(): void {
        deck = null;
        messages = [];
        ready = false;
        phase = "intake";
    }

    function toDecks(): void {
        goto("/speedrun-decks");
    }
</script>

<main>
    <div class="shell">
        <header>
            <h1>Import a deck with AI</h1>
            <button type="button" class="back" onclick={toDecks}>
                <span class="arrow" aria-hidden="true">←</span>
                Back to decks
            </button>
        </header>

        {#if error}
            <p class="error" role="alert">{error}</p>
        {/if}

        {#if phase === "intake"}
            <Intake
                {files}
                {note}
                {extracting}
                {busy}
                {available}
                {hostAvailable}
                onAddFiles={addFiles}
                onRemove={removeFile}
                onNoteChange={(value) => (note = value)}
                onContinue={toChat}
            />
        {:else if phase === "chat"}
            <Chat
                {messages}
                {busy}
                {ready}
                {draft}
                onDraftChange={(value) => (draft = value)}
                onSend={send}
                onGenerate={() => runTurn("generate")}
                onBack={() => (phase = "intake")}
            />
        {:else if deck}
            <DeckPreview
                hierarchy={deck.hierarchy}
                summary={deck.summary}
                {busy}
                onOpenBuilder={openBuilder}
                onBackToChat={() => (phase = "chat")}
                onDiscard={discard}
            />
        {/if}
    </div>
</main>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    main {
        @include sr.tokens;

        min-height: 100vh;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 2.5rem 1.25rem;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
    }
    .shell {
        width: 100%;
        max-width: 680px;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-card);
        box-shadow: var(--sr-shadow-card);
        padding: 1.75rem;
        // Balance the card in the viewport. The chat log and preview tree cap
        // their own height and scroll internally, so the card stays centered;
        // if it ever outgrows the screen the auto margins collapse to the top.
        margin-block: auto;
    }
    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        margin-bottom: 1.5rem;
    }
    h1 {
        margin: 0;
        font-size: 1.4rem;
        font-weight: 600;
    }
    // Ghost back control, matching the account screen: return to the decks home.
    .back {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        flex-shrink: 0;
        padding: 10px 16px;
        border: none;
        border-radius: var(--sr-radius-tile);
        background: var(--sr-ghost);
        color: var(--sr-ink-slate);
        font-family: var(--sr-sans);
        font-weight: 600;
        font-size: 12.5px;
        line-height: 1;
        cursor: pointer;
        transition:
            transform 130ms ease,
            filter 130ms ease;
    }
    .back:hover,
    .back:focus-visible {
        transform: translateY(-1px);
        filter: brightness(1.03);
    }
    .back .arrow {
        font-size: 14px;
    }
    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }
    @media (prefers-reduced-motion: reduce) {
        .back {
            transition: none;
        }
        .back:hover,
        .back:focus-visible {
            transform: none;
        }
    }
    .error {
        margin: 0 0 1rem;
        padding: 0.7rem 0.9rem;
        border-radius: var(--sr-radius-control);
        background: var(--sr-signal-weak);
        border: 1px solid var(--sr-signal-band);
        color: var(--sr-signal-deep);
        font-size: 0.9rem;
    }
</style>
