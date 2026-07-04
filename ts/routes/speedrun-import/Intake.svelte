<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto } from "$app/navigation";

    import type { ExtractedFile } from "./lib";

    const {
        files,
        note,
        extracting,
        busy,
        available,
        hostAvailable,
        onAddFiles,
        onRemove,
        onNoteChange,
        onContinue,
    }: {
        files: ExtractedFile[];
        note: string;
        extracting: boolean;
        busy: boolean;
        available: boolean;
        hostAvailable: boolean;
        onAddFiles: (added: File[]) => void;
        onRemove: (index: number) => void;
        onNoteChange: (value: string) => void;
        onContinue: () => void;
    } = $props();

    let dragging = $state(false);

    const canContinue = $derived(
        hostAvailable &&
            !extracting &&
            !busy &&
            (files.some((file) => file.text.length > 0) || note.trim().length > 0),
    );

    function onDrop(event: DragEvent): void {
        event.preventDefault();
        dragging = false;
        const dropped = event.dataTransfer?.files;
        if (dropped && dropped.length) {
            onAddFiles(Array.from(dropped));
        }
    }

    function onPick(event: Event): void {
        const target = event.currentTarget as HTMLInputElement;
        if (target.files) {
            onAddFiles(Array.from(target.files));
        }
        target.value = "";
    }

    function buildManually(): void {
        goto("/speedrun-hierarchy/new");
    }

    function fileStatus(file: ExtractedFile): string {
        if (file.error) {
            return file.error;
        }
        if (file.truncated) {
            return "shortened to fit";
        }
        return `${file.text.length.toLocaleString()} characters`;
    }
</script>

<div class="intake">
    {#if !hostAvailable}
        <p class="banner warn">
            Open this screen in the Speedrun desktop or phone app to use AI import.
        </p>
    {:else if !available}
        <p class="banner">AI import isn't configured.</p>
    {/if}

    <div
        class="drop"
        class:dragging
        role="group"
        aria-label="Add files"
        ondragover={(event) => {
            event.preventDefault();
            dragging = true;
        }}
        ondragleave={() => (dragging = false)}
        ondrop={onDrop}
    >
        <p class="drop-title">Drop files here</p>
        <p class="drop-types">PDF, TXT, MD, CSV, DOCX</p>
        <label class="choose">
            Choose files
            <input
                class="file-input"
                type="file"
                multiple
                accept=".pdf,.txt,.md,.csv,.docx"
                disabled={busy}
                onchange={onPick}
            />
        </label>
    </div>

    {#if extracting}
        <p class="reading">Reading files…</p>
    {/if}

    {#if files.length}
        <ul class="files">
            {#each files as file, index (file.name + index)}
                <li class:failed={!!file.error}>
                    <div class="file-main">
                        <span class="file-name">{file.name}</span>
                        <span class="file-status">{fileStatus(file)}</span>
                    </div>
                    <button
                        class="remove"
                        onclick={() => onRemove(index)}
                        aria-label={`Remove ${file.name}`}
                    >
                        Remove
                    </button>
                </li>
            {/each}
        </ul>
    {/if}

    <label class="note-field">
        <span>Anything the AI should know? (optional)</span>
        <textarea
            rows="3"
            placeholder="These two PDFs are different sources. Chapter 3 is physics."
            value={note}
            oninput={(event) =>
                onNoteChange((event.currentTarget as HTMLTextAreaElement).value)}
        ></textarea>
    </label>

    <div class="footer">
        <button class="primary" onclick={onContinue} disabled={!canContinue}>
            Continue
        </button>
        <button class="inline" onclick={buildManually}>Build manually</button>
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    .intake {
        @include sr.tokens;

        font-family: var(--sr-sans);
        color: var(--sr-ink);
    }
    .banner {
        margin: 0 0 1rem;
        padding: 0.7rem 0.9rem;
        border-radius: var(--sr-radius-control);
        background: var(--sr-inset);
        border: 1px solid var(--sr-line);
        font-size: 0.9rem;
        color: var(--sr-ink-2);
    }
    .banner.warn {
        background: var(--sr-warn-bg);
        border-color: var(--sr-warn-line);
    }
    .drop {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.4rem;
        padding: 2rem 1rem;
        border: 1.5px dashed var(--sr-line-dashed);
        border-radius: var(--sr-radius-card-sm);
        background: var(--sr-tile);
        text-align: center;
    }
    .drop.dragging {
        border-color: var(--sr-signal);
        background: var(--sr-signal-weak);
    }
    .drop-title {
        margin: 0;
        font-weight: 560;
    }
    .drop-types {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 0.72rem;
        letter-spacing: var(--sr-track-tag);
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .choose {
        position: relative;
        overflow: hidden;
        display: inline-flex;
        align-items: center;
    }
    .choose:focus-within {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
    }
    .file-input {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        opacity: 0;
        cursor: pointer;
    }
    .file-input:disabled {
        cursor: default;
    }
    .reading {
        margin: 0.9rem 0 0;
        font-size: 0.85rem;
        color: var(--sr-ink-2);
    }
    .files {
        list-style: none;
        margin: 1rem 0 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    .files li {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
        padding: 0.55rem 0.75rem;
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-tile);
        background: var(--sr-white);
    }
    .files li.failed {
        border-color: var(--sr-warn-line);
        background: var(--sr-warn-bg);
    }
    .file-main {
        min-width: 0;
        display: flex;
        flex-direction: column;
    }
    .file-name {
        font-weight: 540;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .file-status {
        font-family: var(--sr-mono);
        font-size: 0.7rem;
        letter-spacing: 0.03em;
        color: var(--sr-ink-3);
    }
    .note-field {
        display: block;
        margin: 1.25rem 0 1rem;
    }
    .note-field span {
        display: block;
        margin-bottom: 0.4rem;
        font-size: 0.85rem;
        color: var(--sr-ink-2);
    }
    textarea {
        width: 100%;
        box-sizing: border-box;
        padding: 0.65rem 0.75rem;
        border: 1px solid var(--sr-line-strong);
        border-radius: var(--sr-radius-control);
        background: var(--sr-white);
        color: var(--sr-ink);
        font: inherit;
        resize: vertical;
    }
    button {
        font: inherit;
        cursor: pointer;
    }
    .choose,
    .remove {
        padding: 0.5rem 0.85rem;
        border-radius: var(--sr-radius-control);
        border: 1px solid var(--sr-line-strong);
        background: var(--sr-white);
        color: var(--sr-ink);
        font-weight: 540;
    }
    .remove {
        flex-shrink: 0;
    }
    .footer {
        display: flex;
        align-items: center;
        gap: 1rem;
    }
    .primary {
        padding: 0.65rem 1.1rem;
        border-radius: var(--sr-radius-control);
        border: 1px solid var(--sr-signal);
        background: var(--sr-signal);
        color: var(--sr-white);
        font-weight: 600;
    }
    .primary:disabled {
        opacity: 0.5;
        cursor: default;
    }
    .inline {
        border: none;
        background: none;
        padding: 0;
        color: var(--sr-signal-ink);
        text-decoration: underline;
        font: inherit;
    }
    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
    }
</style>
