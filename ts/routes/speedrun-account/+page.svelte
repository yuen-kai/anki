<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { untrack } from "svelte";

    import { goto } from "$app/navigation";

    import {
        ANKIWEB_SIGNUP_URL,
        hostSignOut,
        hostSyncLogin,
        hostSyncNow,
        hostSyncStatus,
    } from "./lib";
    import type { PageData } from "./$types";

    const { data }: { data: PageData } = $props();

    // Snapshot the load's status once, then let user actions drive local state.
    const initial = untrack(() => data.status);

    // The override keeps the derived host availability reactive to `data`.
    let hostAvailableOverride = $state(false);
    const hostAvailable = $derived(hostAvailableOverride || data.status.hostAvailable);

    let loggedIn = $state(initial.loggedIn);
    let account = $state<string | null>(initial.account);
    // The sync server to sign in against. Pre-filled from whatever the host is
    // already configured for; left blank it means AnkiWeb. Editable on the phone,
    // where there's no separate preferences screen to point at a self-hosted
    // server. Shown read-only once signed in.
    let endpoint = $state(initial.endpoint ?? "");
    let username = $state("");
    let password = $state("");
    let busy = $state(false);
    let message = $state("");
    let error = $state("");
    let lastSynced = $state<string | null>(null);
    // Set when the host reports a full-sync conflict: the server and this device
    // both changed, so the user has to pick which copy to keep (phone only).
    let conflict = $state(false);

    function describe(e: unknown): string {
        return e instanceof Error ? e.message : String(e);
    }

    async function run(action: () => Promise<unknown>, ok = ""): Promise<void> {
        busy = true;
        error = "";
        message = "";
        try {
            await action();
            message = ok;
        } catch (e) {
            error = describe(e);
        } finally {
            busy = false;
        }
    }

    async function doSignIn(): Promise<void> {
        await run(async () => {
            const result = await hostSyncLogin(username.trim(), password, endpoint.trim());
            if (!result.ok) {
                throw new Error(result.message || "Sign in failed.");
            }
            loggedIn = true;
            account = result.account;
            if (result.endpoint) {
                endpoint = result.endpoint;
            }
            password = "";
            hostAvailableOverride = true;
        }, "Signed in.");
    }

    function onSubmit(event: SubmitEvent): void {
        event.preventDefault();
        void doSignIn();
    }

    async function doSync(): Promise<void> {
        conflict = false;
        await run(async () => {
            const res = await hostSyncNow();
            if (res.conflict) {
                conflict = true;
                return;
            }
            if (!res.ok) {
                throw new Error(res.message || "Sync failed.");
            }
            lastSynced = new Date().toLocaleTimeString();
            const status = await hostSyncStatus();
            hostAvailableOverride = hostAvailableOverride || status.hostAvailable;
        }, "Sync complete.");
        // Don't leave a success note above the conflict prompt.
        if (conflict) {
            message = "";
        }
    }

    async function resolveConflict(direction: "download" | "upload"): Promise<void> {
        busy = true;
        error = "";
        message = "";
        try {
            const res = await hostSyncNow(direction);
            if (!res.ok) {
                throw new Error(res.message || "Sync failed.");
            }
            conflict = false;
            lastSynced = new Date().toLocaleTimeString();
            message = res.message;
        } catch (e) {
            error = describe(e);
        } finally {
            busy = false;
        }
    }

    async function doSignOut(): Promise<void> {
        await run(async () => {
            await hostSignOut();
            loggedIn = false;
            account = null;
            lastSynced = null;
            conflict = false;
        });
    }

    function toDecks(): void {
        goto("/speedrun-decks");
    }
</script>

<div class="screen">
    <div class="wrap">
        <nav class="app-nav" aria-label="Speedrun">
            <button type="button" class="app-nav-back" onclick={toDecks}>
                <span class="arrow" aria-hidden="true">←</span>
                Back to decks
            </button>
        </nav>
        <header class="hdr">
            <div class="brand">
                <span class="sr-logo" aria-hidden="true"></span>
                <span class="brand-name">Synapse</span>
                <span class="brand-tag">MCAT</span>
            </div>
            <button type="button" class="sr-btn sr-btn--ghost" onclick={toDecks}>
                <span class="arrow" aria-hidden="true">←</span>
                Back to decks
            </button>
        </header>

        <div class="body">
            <div class="head">
                <div class="sr-eyebrow">Account</div>
                <h1 class="title">{loggedIn ? "Your account" : "Sign in"}</h1>
            </div>

            <section class="sr-card panel">
                {#if !loggedIn}
                    <p class="lede">Sign in to sync your decks across devices.</p>

                    <form class="form" onsubmit={onSubmit}>
                        <label class="field">
                            <span class="label">Email</span>
                            <input
                                class="sr-input"
                                type="text"
                                autocomplete="username"
                                bind:value={username}
                                disabled={busy}
                            />
                        </label>
                        <label class="field">
                            <span class="label">Password</span>
                            <input
                                class="sr-input"
                                type="password"
                                autocomplete="current-password"
                                bind:value={password}
                                disabled={busy}
                            />
                        </label>
                        <label class="field">
                            <span class="label">Sync server</span>
                            <input
                                class="sr-input"
                                type="url"
                                inputmode="url"
                                autocomplete="off"
                                placeholder="Leave blank for AnkiWeb"
                                bind:value={endpoint}
                                disabled={busy}
                            />
                        </label>

                        <button
                            type="submit"
                            class="sr-btn sr-btn--primary full"
                            disabled={busy || !username || !password}
                        >
                            Sign in
                        </button>
                    </form>

                    <a class="create" href={ANKIWEB_SIGNUP_URL}>
                        Create an AnkiWeb account
                    </a>
                {:else}
                    <div class="identity">
                        <span class="label">Signed in as</span>
                        <span class="who">{account ?? "your account"}</span>
                    </div>

                    {#if endpoint}
                        <div class="identity">
                            <span class="label">Server</span>
                            <span class="server">{endpoint}</span>
                        </div>
                    {/if}

                    <button
                        type="button"
                        class="sr-btn sr-btn--primary full"
                        onclick={doSync}
                        disabled={busy}
                    >
                        Sync now
                    </button>

                    {#if conflict}
                        <div class="conflict" role="group" aria-label="Resolve sync conflict">
                            <p class="conflict-msg">
                                This device and the server have both changed since the last
                                sync. Choose which copy to keep. The other copy is replaced.
                            </p>
                            <div class="conflict-actions">
                                <button
                                    type="button"
                                    class="sr-btn sr-btn--primary"
                                    onclick={() => resolveConflict("download")}
                                    disabled={busy}
                                >
                                    Keep server copy
                                </button>
                                <button
                                    type="button"
                                    class="sr-btn sr-btn--ghost"
                                    onclick={() => resolveConflict("upload")}
                                    disabled={busy}
                                >
                                    Keep this device
                                </button>
                            </div>
                        </div>
                    {/if}

                    {#if lastSynced}
                        <p class="synced">Last synced at {lastSynced}.</p>
                    {/if}

                    <button
                        type="button"
                        class="signout"
                        onclick={doSignOut}
                        disabled={busy}
                    >
                        Sign out
                    </button>
                {/if}

                {#if message}
                    <p class="ok" role="status">{message}</p>
                {/if}
                {#if error}
                    <p class="err" role="alert">{error}</p>
                {/if}
            </section>

            {#if !hostAvailable}
                <p class="hint">
                    Open this screen in the Speedrun desktop or phone app to sign in and
                    sync. A plain browser has no sync host.
                </p>
            {/if}
        </div>
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

    .wrap {
        max-width: 1240px;
        margin: 0 auto;
        // Fill the screen so the short card can center in the space below the
        // header instead of stranding all the whitespace beneath it.
        min-height: 100vh;
        display: flex;
        flex-direction: column;
    }

    // Phone-only top nav: the desktop header (and its Back to decks control) is
    // hidden at phone width, so surface the return to decks here as a >=44px
    // touch target. Mirrors the mobile app-nav on the decks home.
    .app-nav {
        display: none;
        align-items: center;
        padding: 8px 14px;
        border-bottom: 1px solid var(--sr-line-2);
    }
    .app-nav-back {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        min-height: 44px;
        padding: 8px 12px;
        border: none;
        border-radius: var(--sr-radius-control);
        background: none;
        color: var(--sr-ink-2);
        font-family: var(--sr-sans);
        font-weight: 540;
        font-size: 13px;
        cursor: pointer;
    }
    .app-nav-back:hover {
        background: var(--sr-ghost);
        color: var(--sr-ink);
    }
    .app-nav-back .arrow {
        font-size: 14px;
    }

    // Header: brand mark + wordmark on the left, return to decks on the right.
    .hdr {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
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
    .arrow {
        font-size: 14px;
    }

    // A single readable column, centered in the full-width page. margin:auto
    // also balances the vertical space below the header; if the content ever
    // outgrows the viewport the auto margins collapse and it flows from the top.
    .body {
        box-sizing: border-box;
        width: 100%;
        max-width: 460px;
        margin: auto;
        padding: 52px 24px 72px;
    }

    .head {
        margin-bottom: 22px;
    }
    .title {
        margin: 4px 0 0;
        font-size: 28px;
        font-weight: 700;
        letter-spacing: var(--sr-tighten);
        color: var(--sr-ink);
    }

    .panel {
        display: flex;
        flex-direction: column;
        gap: 16px;
        padding: 26px;
    }

    .lede {
        margin: 0;
        color: var(--sr-ink-2);
        font-size: 14px;
    }

    .form {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .label {
        font-family: var(--sr-mono);
        font-weight: 500;
        font-size: 10px;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }

    .full {
        width: 100%;
    }

    .identity {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    .who {
        font-size: 16px;
        font-weight: 600;
        color: var(--sr-ink);
        word-break: break-word;
    }
    .server {
        font-family: var(--sr-mono);
        font-size: 13px;
        color: var(--sr-ink-2);
        word-break: break-all;
    }

    .synced {
        margin: 0;
        font-size: 13px;
        color: var(--sr-stage-solo-deep);
    }

    .conflict {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 14px 16px;
        border: 1px solid var(--sr-line-2);
        border-radius: var(--sr-radius-control);
        background: var(--sr-ghost);
    }
    .conflict-msg {
        margin: 0;
        font-size: 13px;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }
    .conflict-actions {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
    }
    .conflict-actions :global(.sr-btn) {
        flex: 1 1 0;
        min-width: 8rem;
    }

    .signout {
        align-self: flex-start;
        padding: 6px 0;
        border: none;
        background: none;
        color: var(--sr-ink-2);
        font-family: var(--sr-sans);
        font-size: 13px;
        text-decoration: underline;
        text-underline-offset: 3px;
        cursor: pointer;
    }
    .signout:hover {
        color: var(--sr-ink);
    }

    // Secondary path for new users: sits under Sign in. Rendered as a plain
    // link so the embedded webviews open it in the system browser.
    .create {
        align-self: center;
        color: var(--sr-ink-2);
        font-size: 13px;
        text-decoration: underline;
        text-underline-offset: 3px;
    }
    .create:hover {
        color: var(--sr-ink);
    }

    .ok {
        margin: 0;
        color: var(--sr-stage-solo-deep);
        font-size: 13px;
    }
    .err {
        margin: 0;
        color: var(--sr-signal-deep);
        font-size: 13px;
    }

    .hint {
        margin: 16px 0 0;
        color: var(--sr-ink-3);
        font-size: 13px;
        line-height: 1.5;
    }

    button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }

    // Phone: the desktop brand bar is OS/shell chrome, so it drops away and the
    // content leads in a single column with comfortable touch targets.
    @media (max-width: $phone) {
        .hdr {
            display: none;
        }
        // Surface the phone back nav where the desktop header is hidden.
        .app-nav {
            display: flex;
        }
        // Lead from the top under the back nav instead of floating the card in
        // the middle of a tall phone screen.
        .body {
            max-width: 100%;
            margin-block: 0;
            padding: 24px 16px 48px;
        }
        .title {
            font-size: 22px;
        }
        .panel :global(.sr-btn) {
            min-height: 44px;
        }
        .signout,
        .create {
            min-height: 44px;
            display: inline-flex;
            align-items: center;
        }
    }
</style>
