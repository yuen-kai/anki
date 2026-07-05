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

export interface HostSyncResult {
    ok: boolean;
    message: string;
    // Set by the phone host when a full sync is required in an ambiguous
    // direction, so the screen can ask the user which copy to keep. The desktop
    // resolves this with its own dialog and never reports a conflict here.
    conflict?: boolean;
    canUpload?: boolean;
    canDownload?: boolean;
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

/**
 * Ask the host to run a collection sync now. Pass a `resolve` direction to force
 * one side of a full sync after the host reported a conflict (phone only).
 */
export async function hostSyncNow(resolve?: "upload" | "download"): Promise<HostSyncResult> {
    return dec<HostSyncResult>(await speedrunSyncNow({ json: enc(resolve ? { resolve } : {}) }, quiet));
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
