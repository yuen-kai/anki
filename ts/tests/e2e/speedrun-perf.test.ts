// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// Speedrun desktop performance harness. Drives the real Svelte app served by a
// (reused) mediasrv instance and records input->paint latency, next-card
// latency, dashboard load, and main-thread long tasks (jank). Results are
// written as JSON under out/speedrun_perf/ for the report.
//
// Run against a pre-launched instance (ANKI_E2E_REUSE_SERVER=1). Select the
// review deck via SR_PERF_DECK_ID. See qt/tests/speedrun_perf_rpc.py for seeding
// and deck ids.

import type { Page } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";

import { expect, test } from "./fixtures";

const N = Number(process.env.SR_PERF_N ?? "60");
const LABEL = process.env.SR_PERF_LABEL ?? "deck";
const DECK_ID = process.env.SR_PERF_DECK_ID ?? "";
const OUT_DIR = "out/speedrun_perf";

function pctile(xs: number[], p: number): number {
    if (xs.length === 0) { return NaN; }
    const s = [...xs].sort((a, b) => a - b);
    const k = (s.length - 1) * p;
    const lo = Math.floor(k);
    const hi = Math.min(lo + 1, s.length - 1);
    return s[lo] + (s[hi] - s[lo]) * (k - lo);
}

function summarize(name: string, xs: number[]): Record<string, number> {
    const r = {
        n: xs.length,
        p50: +pctile(xs, 0.5).toFixed(2),
        p95: +pctile(xs, 0.95).toFixed(2),
        max: +Math.max(...xs).toFixed(2),
        min: +Math.min(...xs).toFixed(2),
    };
    // eslint-disable-next-line no-console
    console.log(`[perf] ${name} ${JSON.stringify(r)}`);
    return r;
}

function writeResult(label: string, data: unknown): void {
    mkdirSync(OUT_DIR, { recursive: true });
    const path = `${OUT_DIR}/fe_${label}.json`;
    writeFileSync(path, JSON.stringify(data, null, 2));
    // eslint-disable-next-line no-console
    console.log(`[perf] wrote ${path}`);
}

// Injected into every page: long-task + long-animation-frame + event-timing
// observers, all pushing into window.__perf so the test can read worst-case
// main-thread blocking (target 8) after each scenario.
const INSTRUMENT = () => {
    interface Perf {
        longtasks: number[];
        loaf: number[];
        events: { name: string; duration: number }[];
    }
    const w = window as unknown as { __perf: Perf; speedrunPlatform?: string };
    w.__perf = { longtasks: [], loaf: [], events: [] };
    try {
        new PerformanceObserver((l) => {
            for (const e of l.getEntries()) { w.__perf.longtasks.push(e.duration); }
        }).observe({ type: "longtask", buffered: true });
    } catch { /* unsupported */ }
    try {
        new PerformanceObserver((l) => {
            for (const e of l.getEntries()) { w.__perf.loaf.push(e.duration); }
        }).observe({ type: "long-animation-frame", buffered: true });
    } catch { /* unsupported */ }
    try {
        new PerformanceObserver((l) => {
            for (const e of l.getEntries() as PerformanceEventTiming[]) {
                w.__perf.events.push({ name: e.name, duration: e.duration });
            }
        }).observe({ type: "event", durationThreshold: 16, buffered: true } as PerformanceObserverInit);
    } catch { /* unsupported */ }
};

// Sanctioned phone proxy: throttle the renderer CPU (CDP) to approximate a
// mid-range phone's UI thread. Note this slows browser JS/paint only, not the
// desktop Rust backend, so it proxies UI latency/jank, not on-device RPC time.
const CPU_THROTTLE = Number(process.env.SR_PERF_CPU_THROTTLE ?? "0");

async function maybeThrottle(page: Page): Promise<void> {
    if (CPU_THROTTLE > 1) {
        const client = await page.context().newCDPSession(page);
        await client.send("Emulation.setCPUThrottlingRate", { rate: CPU_THROTTLE });
    }
}

async function resetPerf(page: Page): Promise<void> {
    await page.evaluate(() => {
        const w = window as unknown as { __perf?: { longtasks: number[]; loaf: number[]; events: unknown[] } };
        if (w.__perf) {
            w.__perf.longtasks = [];
            w.__perf.loaf = [];
            w.__perf.events = [];
        }
    });
}

async function readJank(
    page: Page,
): Promise<{ longtasks: number[]; loaf: number[]; events: { name: string; duration: number }[] }> {
    return page.evaluate(() => {
        const w = window as unknown as {
            __perf: { longtasks: number[]; loaf: number[]; events: { name: string; duration: number }[] };
        };
        return w.__perf;
    });
}

test.describe("speedrun desktop perf", () => {
    test("review latency: ack + next-card + jank", async ({ page }) => {
        test.skip(!DECK_ID, "set SR_PERF_DECK_ID");
        test.setTimeout(600_000);
        await page.addInitScript(() => {
            (window as unknown as { speedrunPlatform?: string }).speedrunPlatform = "mobile";
        });
        await page.addInitScript(INSTRUMENT);
        await maybeThrottle(page);

        await page.goto(`/speedrun-review/${DECK_ID}`);

        const check = page.locator(".check-btn"); // PracticeRecall reveal
        const again = page.getByRole("button", { name: "Again", exact: true });
        const proceed = page.locator(".proceed"); // "Next card →"
        const done = page.getByText("Nothing due right now");

        const ack: number[] = [];
        const next: number[] = [];

        // Wait for the first practice card.
        await check.first().waitFor({ state: "visible", timeout: 60_000 });

        for (let i = 0; i < N; i++) {
            if (await done.isVisible().catch(() => false)) { break; }
            // Reveal the grade bar.
            await check.first().waitFor({ state: "visible", timeout: 30_000 });
            await check.first().click();
            await again.first().waitFor({ state: "visible", timeout: 30_000 });

            // ---- Target 1: input -> acknowledged (grade selected) paint ----
            await page.evaluate(() => {
                const w = window as unknown as { __ack: Promise<number> };
                w.__ack = new Promise<number>((res) => {
                    const h = (e: Event) => {
                        const t = e.target as HTMLElement;
                        if (t && t.closest(".grade")) {
                            const t0 = performance.now();
                            document.removeEventListener("click", h, true);
                            requestAnimationFrame(() => requestAnimationFrame(() => res(performance.now() - t0)));
                        }
                    };
                    document.addEventListener("click", h, true);
                });
            });
            await again.first().click();
            ack.push(await page.evaluate(() => (window as unknown as { __ack: Promise<number> }).__ack));

            // ---- Target 2: click "Next card" -> next card visible ----
            await proceed.first().waitFor({ state: "visible", timeout: 30_000 });
            await page.evaluate(() => {
                const w = window as unknown as { __next: Promise<number> };
                w.__next = new Promise<number>((res) => {
                    const h = (e: Event) => {
                        const t = e.target as HTMLElement;
                        if (t && t.closest(".proceed")) {
                            const t0 = performance.now();
                            document.removeEventListener("click", h, true);
                            const tick = () => {
                                if (
                                    document.querySelector(".check-btn")
                                    || document.querySelector(".message-title")
                                ) {
                                    res(performance.now() - t0);
                                } else {
                                    requestAnimationFrame(tick);
                                }
                            };
                            requestAnimationFrame(tick);
                        }
                    };
                    document.addEventListener("click", h, true);
                });
            });
            await proceed.first().click();
            next.push(await page.evaluate(() => (window as unknown as { __next: Promise<number> }).__next));
        }

        const jank = await readJank(page);
        // speedrunNextCard round-trip as seen by the browser (server work + IPC).
        const rpc = await page.evaluate(() =>
            performance.getEntriesByType("resource")
                .filter((e) => e.name.includes("speedrunNextCard"))
                .map((e) => e.duration)
        );

        const result = {
            label: LABEL,
            deckId: DECK_ID,
            ack_ms: summarize(`ack (${LABEL})`, ack),
            next_ms: summarize(`next (${LABEL})`, next),
            nextcard_rpc_ms: rpc.length ? summarize(`nextcard-rpc (${LABEL})`, rpc) : null,
            longtask_max_ms: jank.longtasks.length ? +Math.max(...jank.longtasks).toFixed(2) : 0,
            longtask_count: jank.longtasks.length,
            loaf_max_ms: jank.loaf.length ? +Math.max(...jank.loaf).toFixed(2) : 0,
            slow_clicks: jank.events.filter((e) => /click|keydown|pointerdown/.test(e.name)),
        };
        writeResult(`review_${LABEL}`, result);
        expect(ack.length).toBeGreaterThan(0);
    });

    test("dashboard: first load (cold) + refresh + jank", async ({ browser }) => {
        test.setTimeout(600_000);
        // Desktop dashboard path: NO mobile shell (matches Qt full-page load,
        // which skips the mobile ensureSeeded step).
        const firstLoad: number[] = [];
        const refresh: number[] = [];
        const refreshLongtaskMax: number[] = [];

        // Cold first-load: a brand-new context per sample (empty HTTP cache),
        // measure time from navigation start to first deck row painted.
        const cold = Number(process.env.SR_PERF_DASH_COLD ?? "50");
        for (let i = 0; i < cold; i++) {
            const ctx = await browser.newContext();
            const p = await ctx.newPage();
            await p.addInitScript(INSTRUMENT);
            await maybeThrottle(p);
            await p.goto("/speedrun-decks", { waitUntil: "commit" });
            const t = await p.evaluate(async () => {
                const start = performance.now();
                await new Promise<void>((res) => {
                    const tick = () => {
                        if (document.querySelector("article")) { res(); }
                        else { requestAnimationFrame(tick); }
                    };
                    requestAnimationFrame(tick);
                });
                // time since navigation start (performance.now is nav-relative)
                void start;
                return performance.now();
            });
            firstLoad.push(t);
            await ctx.close();
        }

        // Warm refresh (reload same page/context) + long-task capture.
        const warm = await browser.newContext();
        const wp = await warm.newPage();
        await wp.addInitScript(INSTRUMENT);
        await maybeThrottle(wp);
        await wp.goto("/speedrun-decks");
        await wp.locator("article").first().waitFor({ state: "visible", timeout: 30_000 });
        const rounds = Number(process.env.SR_PERF_DASH_REFRESH ?? "50");
        for (let i = 0; i < rounds; i++) {
            await resetPerf(wp);
            await wp.reload({ waitUntil: "commit" });
            const t = await wp.evaluate(async () => {
                await new Promise<void>((res) => {
                    const tick = () => {
                        if (document.querySelector("article")) { res(); }
                        else { requestAnimationFrame(tick); }
                    };
                    requestAnimationFrame(tick);
                });
                return performance.now();
            });
            refresh.push(t);
            const jank = await readJank(wp);
            refreshLongtaskMax.push(jank.longtasks.length ? Math.max(...jank.longtasks) : 0);
        }
        await warm.close();

        const result = {
            first_load_ms: summarize("dashboard first-load (cold)", firstLoad),
            refresh_ms: summarize("dashboard refresh (warm)", refresh),
            refresh_longtask_max_ms: +Math.max(...refreshLongtaskMax, 0).toFixed(2),
        };
        writeResult(`dashboard_${LABEL}`, result);
        expect(firstLoad.length).toBeGreaterThan(0);
    });
});
