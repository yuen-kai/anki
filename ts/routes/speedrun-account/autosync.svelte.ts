// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// Periodic auto-sync for the Speedrun shell (desktop Qt / Android). A single
// timer, started once from a Speedrun screen, runs a quiet two-way sync every
// few minutes and whenever the app returns to the foreground. It never shows a
// progress spinner or a conflict dialog: if the collections have diverged so a
// one-way full sync is needed, it flags `needsAttention` and waits for the user
// to resolve it via the interactive "Sync now" button. Manual sync stays fully
// interactive and untouched.

import { hostAutoSync } from "./lib";

const INTERVAL_MS = 5 * 60 * 1000;
const INITIAL_DELAY_MS = 10 * 1000;
// Don't re-sync on every focus flip; only if this long has passed since the last try.
const FOREGROUND_MIN_GAP_MS = 60 * 1000;

let needsAttention = $state(false);
let lastSyncedAt = $state<number | null>(null);

// Non-reactive control state.
let started = false;
let inFlight = false;
let lastAttempt = 0;

/** Reactive view of auto-sync state for the UI (getters track the runes). */
export const autoSyncState = {
    get needsAttention(): boolean {
        return needsAttention;
    },
    get lastSyncedAt(): number | null {
        return lastSyncedAt;
    },
};

/** Clear the "needs attention" flag, e.g. after a manual sync resolves the conflict. */
export function clearAutoSyncAttention(): void {
    needsAttention = false;
}

async function runOnce(): Promise<void> {
    if (inFlight) {
        return;
    }
    inFlight = true;
    lastAttempt = Date.now();
    try {
        const res = await hostAutoSync();
        if (res.status === "synced") {
            needsAttention = false;
            lastSyncedAt = Date.now();
        } else if (res.status === "conflict") {
            needsAttention = true;
        }
        // not-signed-in / offline / error: leave state as-is and retry next tick.
    } catch {
        // Never let a background failure surface.
    } finally {
        inFlight = false;
    }
}

/** Trigger an immediate quiet sync (fire-and-forget). */
export function autoSyncNow(): void {
    void runOnce();
}

/** Start the periodic auto-sync. Idempotent: safe to call from every screen's mount. */
export function startAutoSync(): void {
    if (started || typeof window === "undefined") {
        return;
    }
    started = true;
    window.setTimeout(runOnce, INITIAL_DELAY_MS);
    window.setInterval(runOnce, INTERVAL_MS);
    document.addEventListener("visibilitychange", () => {
        if (
            document.visibilityState === "visible"
            && Date.now() - lastAttempt > FOREGROUND_MIN_GAP_MS
        ) {
            void runOnce();
        }
    });
}
