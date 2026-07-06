// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// Bridge from the account screen to the host (desktop Qt / Android). These call
// the Qt-only Speedrun account/sync RPCs, which drive Anki's own native sync
// login against the in-repo self-hosted server (SimpleServer) with a username +
// password. The handlers live in qt/aqt/mediasrv.py (speedrun_sync_login /
// _sync_now / _sign_out / _sync_status) and are declared on FrontendService in
// proto/anki/frontend.proto, so the generated bindings below handle the
// protobuf `Json` wrapping, the `application/binary` content type, and the auth
// header the host requires (a raw JSON fetch is rejected with a 403). This
// mirrors the shared plumbing in ../speedrun-hierarchy/lib.ts.

import { speedrunSignOut, speedrunSyncLogin, speedrunSyncNow, speedrunSyncStatus } from "@generated/backend";

// AnkiWeb's account registration page. Rendered as a plain link on the sign-in
// form: the embedded webviews (desktop Qt / Android) intercept the navigation
// to this remote URL and open it in the system browser, while a plain browser
// just follows the link.
export const ANKIWEB_SIGNUP_URL = "https://ankiweb.net/account/signup";

export interface HostSyncStatus {
    // Whether the host currently holds sync credentials.
    loggedIn: boolean;
    // The signed-in account's username, if known.
    account: string | null;
    // The sync server endpoint the host will sync against.
    endpoint: string | null;
    // Whether the host bridge is available at all (false in a plain browser).
    hostAvailable: boolean;
}

export interface HostLoginResult {
    ok: boolean;
    account: string | null;
    endpoint: string | null;
    message?: string;
}

// Outcome of a quiet (periodic) auto-sync. "synced" = a normal two-way merge
// completed (or nothing to do); "conflict" = the collections have diverged and
// a one-way full sync is needed, which auto-sync deliberately does not perform
// (the user resolves it with the interactive "Sync now" button).
export type AutoSyncStatus =
    | "synced"
    | "conflict"
    | "not-signed-in"
    | "offline"
    | "error";

export interface HostAutoSyncResult {
    ok: boolean;
    status: AutoSyncStatus;
    message?: string;
}

// The Speedrun RPCs exchange a `{ json }` blob (a protobuf generic.Json wrapper);
// these mirror the encode/decode helpers ../speedrun-hierarchy/lib.ts uses.
const enc = (value: unknown): Uint8Array => new TextEncoder().encode(JSON.stringify(value));
const dec = <T>(reply: { json: Uint8Array }): T => JSON.parse(new TextDecoder().decode(reply.json)) as T;

// Every call site reports failures inline, so opt out of the global dialog.
const quiet = { alertOnError: false } as const;

/** Log in to the self-hosted sync server with a username and password. */
export async function hostSyncLogin(
    username: string,
    password: string,
    endpoint: string,
): Promise<HostLoginResult> {
    return dec<HostLoginResult>(
        await speedrunSyncLogin({ json: enc({ username, password, endpoint }) }, quiet),
    );
}

/** Ask the host to run a collection sync now (interactive: progress + conflict UI). */
export async function hostSyncNow(): Promise<{ ok: boolean; message: string }> {
    return dec<{ ok: boolean; message: string }>(await speedrunSyncNow({ json: enc({}) }, quiet));
}

/**
 * Quiet background sync for the periodic auto-sync. Runs only a normal two-way
 * merge; if a one-way full sync is required it reports "conflict" and does
 * nothing, leaving resolution to the interactive hostSyncNow(). Never shows UI.
 */
export async function hostAutoSync(): Promise<HostAutoSyncResult> {
    try {
        return dec<HostAutoSyncResult>(
            await speedrunSyncNow({ json: enc({ interactive: false }) }, quiet),
        );
    } catch {
        return { ok: false, status: "offline" };
    }
}

/** Tell the host to drop its sync credentials (sign out). */
export async function hostSignOut(): Promise<{ ok: boolean }> {
    return dec<{ ok: boolean }>(await speedrunSignOut({ json: enc({}) }, quiet));
}

/** Current host sync status; reports hostAvailable=false when there's no host. */
export async function hostSyncStatus(): Promise<HostSyncStatus> {
    try {
        return dec<HostSyncStatus>(await speedrunSyncStatus({ json: enc({}) }, quiet));
    } catch {
        return { loggedIn: false, account: null, endpoint: null, hostAvailable: false };
    }
}
