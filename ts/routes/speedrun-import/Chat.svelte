<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { ChatMsg } from "./lib";

    const {
        messages,
        busy,
        ready,
        draft,
        onDraftChange,
        onSend,
        onGenerate,
        onBack,
    }: {
        messages: ChatMsg[];
        busy: boolean;
        ready: boolean;
        draft: string;
        onDraftChange: (value: string) => void;
        onSend: () => void;
        onGenerate: () => void;
        onBack: () => void;
    } = $props();

    // Keep the newest message in view as the conversation grows; reading the
    // reactive counts re-runs the effect on each new turn.
    let logEl = $state<HTMLDivElement>();
    $effect(() => {
        void messages.length;
        void busy;
        if (logEl) {
            logEl.scrollTop = logEl.scrollHeight;
        }
    });

    function onKeydown(event: KeyboardEvent): void {
        if (event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            if (!busy && draft.trim()) {
                onSend();
            }
        }
    }
</script>

<div class="chat">
    <div class="log" bind:this={logEl}>
        {#each messages as message, index (index)}
            <div class="turn" class:user={message.role === "user"}>
                <span class="who">{message.role === "user" ? "You" : "Assistant"}</span>
                <p class="bubble">{message.content}</p>
            </div>
        {/each}
        {#if busy}
            <div class="turn">
                <span class="who">Assistant</span>
                <p class="bubble thinking" aria-live="polite">Thinking…</p>
            </div>
        {/if}
    </div>

    {#if ready}
        <p class="ready" role="status">Ready to build a draft when you are.</p>
    {/if}

    <label class="composer">
        <span class="sr-only">Your reply</span>
        <textarea
            rows="2"
            placeholder="Answer, or add detail…"
            value={draft}
            disabled={busy}
            onkeydown={onKeydown}
            oninput={(event) =>
                onDraftChange((event.currentTarget as HTMLTextAreaElement).value)}
        ></textarea>
        <button class="send" onclick={onSend} disabled={busy || !draft.trim()}>
            Send
        </button>
    </label>

    <div class="actions">
        <button class="link" onclick={onBack} disabled={busy}>Back to files</button>
        <button
            class="generate"
            class:primary={ready}
            onclick={onGenerate}
            disabled={busy}
        >
            Generate deck
        </button>
    </div>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    .chat {
        @include sr.tokens;

        display: flex;
        flex-direction: column;
        font-family: var(--sr-sans);
        color: var(--sr-ink);
    }
    .log {
        display: flex;
        flex-direction: column;
        gap: 0.85rem;
        max-height: 46vh;
        overflow-y: auto;
        padding: 0.25rem;
    }
    .turn {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        max-width: 85%;
    }
    .turn.user {
        align-self: flex-end;
        align-items: flex-end;
    }
    .who {
        font-family: var(--sr-mono);
        font-size: 0.66rem;
        letter-spacing: var(--sr-track-tag);
        text-transform: uppercase;
        color: var(--sr-faint);
        margin-bottom: 0.25rem;
    }
    .bubble {
        margin: 0;
        padding: 0.65rem 0.85rem;
        border-radius: var(--sr-radius-node);
        background: var(--sr-panel-2);
        border: 1px solid var(--sr-line);
        white-space: pre-wrap;
        line-height: 1.45;
    }
    .turn.user .bubble {
        background: var(--sr-signal-weak);
        border-color: var(--sr-signal-band);
        color: var(--sr-signal-deep);
    }
    .thinking {
        color: var(--sr-ink-3);
    }
    .ready {
        margin: 0.9rem 0 0.4rem;
        font-size: 0.85rem;
        color: var(--sr-stage-solo-deep);
    }
    .composer {
        display: flex;
        gap: 0.6rem;
        align-items: flex-end;
        margin-top: 1rem;
    }
    textarea {
        flex: 1;
        box-sizing: border-box;
        padding: 0.6rem 0.7rem;
        border: 1px solid var(--sr-line-strong);
        border-radius: var(--sr-radius-control);
        background: var(--sr-white);
        color: var(--sr-ink);
        font: inherit;
        resize: vertical;
    }
    .actions {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-top: 1rem;
    }
    button {
        font: inherit;
        cursor: pointer;
    }
    .send,
    .generate {
        padding: 0.55rem 0.95rem;
        border-radius: var(--sr-radius-control);
        border: 1px solid var(--sr-line-strong);
        background: var(--sr-white);
        color: var(--sr-ink);
        font-weight: 560;
    }
    .primary {
        background: var(--sr-signal);
        border-color: var(--sr-signal);
        color: var(--sr-white);
    }
    button:disabled {
        opacity: 0.5;
        cursor: default;
    }
    .link {
        border: none;
        background: none;
        padding: 0;
        color: var(--sr-ink-2);
        text-decoration: underline;
        font-weight: 500;
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
    }
</style>
