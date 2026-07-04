// Synapse visual QA capture. Usage: node capture.mjs <round>
import { chromium } from "@playwright/test";
import { mkdirSync } from "node:fs";

const ROUND = process.argv[2] ?? "1";
const BASE = `/tmp/synapse-qa/round-${ROUND}/web`;
mkdirSync(BASE, { recursive: true });

const ORIGIN = "http://127.0.0.1:40000";

// Encode a JS object as an anki.generic.Json protobuf message (field 1, bytes),
// so we can stub the `{ json }` RPC replies at the network layer.
function jsonProto(obj) {
    const payload = new TextEncoder().encode(JSON.stringify(obj));
    const varint = [];
    let n = payload.length;
    do {
        let b = n & 0x7f;
        n >>>= 7;
        if (n) b |= 0x80;
        varint.push(b);
    } while (n);
    return Buffer.from([0x0a, ...varint, ...payload]);
}
const fulfillJson = (obj) => ({
    status: 200,
    contentType: "application/binary",
    body: jsonProto(obj),
});

const DESKTOP = { width: 1280, height: 900 };
const PHONE = { width: 390, height: 844 };

async function shot(page, name, { full = false } = {}) {
    const path = `${BASE}/${name}.png`;
    try {
        await page.screenshot({ path, fullPage: full });
        console.log("  saved", name);
    } catch (e) {
        console.log("  FAILED", name, String(e).split("\n")[0]);
    }
}

const settle = (page, ms = 500) => page.waitForTimeout(ms);

async function run() {
    const browser = await chromium.launch();

    // ---- Desktop context (no mobile shell flag; Qt-style header visible) ----
    const desktop = await browser.newContext({
        viewport: DESKTOP,
        deviceScaleFactor: 2,
    });
    // ---- Phone context (mobile shell: client-side nav + app-nav) ----
    const phone = await browser.newContext({
        viewport: PHONE,
        deviceScaleFactor: 2,
        isMobile: true,
        hasTouch: true,
    });
    await phone.addInitScript(() => {
        window.speedrunPlatform = "mobile";
    });

    // -- Discover a real deckId via the phone (client-side nav) study flow --
    let deckId = null;
    try {
        const p = await phone.newPage();
        await p.goto(`${ORIGIN}/speedrun-decks`, { waitUntil: "networkidle" });
        await p.getByRole("article").first().waitFor({ timeout: 15000 });
        await p.getByRole("button", { name: "Study" }).first().click();
        await p.waitForURL(/speedrun-study\/\d+/, { timeout: 20000 });
        deckId = p.url().match(/speedrun-study\/(\d+)/)[1];
        console.log("deckId:", deckId);
        await p.close();
    } catch (e) {
        console.log("deckId discovery failed:", String(e).split("\n")[0]);
    }

    for (const [device, ctx] of [["desktop", desktop], ["phone", phone]]) {
        console.log(`\n=== ${device} ===`);

        // ---- Decks: populated ----
        try {
            const page = await ctx.newPage();
            await page.goto(`${ORIGIN}/speedrun-decks`, { waitUntil: "networkidle" });
            await settle(page);
            await shot(page, `decks-${device}`);
            await shot(page, `decks-${device}-full`, { full: true });

            // Delete confirm interaction
            try {
                await page.getByRole("button", { name: "Delete deck" }).first().click();
                await settle(page, 300);
                await shot(page, `decks-${device}-confirm`);
            } catch (e) {
                console.log("  confirm skip:", String(e).split("\n")[0]);
            }
            await page.close();
        } catch (e) {
            console.log("decks failed:", String(e).split("\n")[0]);
        }

        // ---- Decks: empty (stub list -> []) ----
        try {
            const page = await ctx.newPage();
            await page.route("**/_anki/speedrunListDecks", (r) => r.fulfill(fulfillJson([])));
            await page.goto(`${ORIGIN}/speedrun-decks`, { waitUntil: "networkidle" });
            await settle(page);
            await shot(page, `decks-${device}-empty`);
            await shot(page, `decks-${device}-empty-full`, { full: true });
            await page.close();
        } catch (e) {
            console.log("decks-empty failed:", String(e).split("\n")[0]);
        }

        // ---- Account: signed-out (stub status) ----
        try {
            const page = await ctx.newPage();
            await page.route("**/_anki/speedrunSyncStatus", (r) =>
                r.fulfill(fulfillJson({ loggedIn: false, account: null, endpoint: null, hostAvailable: true })));
            await page.goto(`${ORIGIN}/speedrun-account`, { waitUntil: "networkidle" });
            await settle(page);
            await shot(page, `account-${device}-signedout`);
            await shot(page, `account-${device}-signedout-full`, { full: true });
            await page.close();
        } catch (e) {
            console.log("account-out failed:", String(e).split("\n")[0]);
        }

        // ---- Account: signed-in (stub status) ----
        try {
            const page = await ctx.newPage();
            await page.route("**/_anki/speedrunSyncStatus", (r) =>
                r.fulfill(fulfillJson({ loggedIn: true, account: "test@example.com", endpoint: "", hostAvailable: true })));
            await page.goto(`${ORIGIN}/speedrun-account`, { waitUntil: "networkidle" });
            await settle(page);
            await shot(page, `account-${device}-signedin`);
            await shot(page, `account-${device}-signedin-full`, { full: true });
            await page.close();
        } catch (e) {
            console.log("account-in failed:", String(e).split("\n")[0]);
        }

        // ---- Import: intake ----
        try {
            const page = await ctx.newPage();
            await page.goto(`${ORIGIN}/speedrun-import`, { waitUntil: "networkidle" });
            await settle(page);
            await shot(page, `import-${device}`);
            await shot(page, `import-${device}-full`, { full: true });
            await page.close();
        } catch (e) {
            console.log("import failed:", String(e).split("\n")[0]);
        }

        if (deckId) {
            // ---- Study overview ----
            try {
                const page = await ctx.newPage();
                await page.goto(`${ORIGIN}/speedrun-study/${deckId}`, { waitUntil: "networkidle" });
                await settle(page, 800);
                await shot(page, `study-${device}`);
                await shot(page, `study-${device}-full`, { full: true });
                await page.close();
            } catch (e) {
                console.log("study failed:", String(e).split("\n")[0]);
            }

            // ---- Hierarchy builder ----
            try {
                const page = await ctx.newPage();
                await page.goto(`${ORIGIN}/speedrun-hierarchy/${deckId}`, { waitUntil: "networkidle" });
                await settle(page, 800);
                await shot(page, `hierarchy-${device}`);
                await shot(page, `hierarchy-${device}-full`, { full: true });
                await page.close();
            } catch (e) {
                console.log("hierarchy failed:", String(e).split("\n")[0]);
            }

            // ---- Review session: capture intro / first card ----
            try {
                const page = await ctx.newPage();
                await page.goto(`${ORIGIN}/speedrun-review/${deckId}`, { waitUntil: "networkidle" });
                await settle(page, 800);
                await shot(page, `review-${device}`);
                await shot(page, `review-${device}-full`, { full: true });
                // Try to step through Begin -> learn
                try {
                    await page.getByRole("button", { name: "Begin" }).first().click({ timeout: 4000 });
                    await settle(page, 700);
                    await shot(page, `review-${device}-after-begin`);
                } catch { /* not on intro */ }
                await page.close();
            } catch (e) {
                console.log("review failed:", String(e).split("\n")[0]);
            }
        }

        // ---- Review demo gallery + key scenes ----
        try {
            const page = await ctx.newPage();
            await page.goto(`${ORIGIN}/speedrun-review-demo`, { waitUntil: "networkidle" });
            await settle(page);
            await shot(page, `demo-${device}`);
            const scenes = [
                "Builder", "Concept editor", "Study \u2014 not started", "New topic",
                "Learn", "Topic learned", "Practice", "Guided", "Solo", "Session complete",
            ];
            for (const label of scenes) {
                try {
                    await page.getByRole("button", { name: label, exact: true }).click({ timeout: 4000 });
                    await settle(page, 700);
                    const slug = label.toLowerCase().replace(/[^a-z]+/g, "-").replace(/^-|-$/g, "");
                    await shot(page, `demo-${device}-${slug}`);
                } catch (e) {
                    console.log(`  demo scene ${label} skip:`, String(e).split("\n")[0]);
                }
            }
            await page.close();
        } catch (e) {
            console.log("demo failed:", String(e).split("\n")[0]);
        }
    }

    await browser.close();
    console.log("\nDONE round", ROUND);
}

run().catch((e) => {
    console.error(e);
    process.exit(1);
});
