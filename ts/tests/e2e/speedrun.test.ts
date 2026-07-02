// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// End-to-end walk of the authored Speedrun study flow against a real, freshly
// seeded collection (the launcher auto-seeds the "MCAT Biochemistry (demo)"
// authored deck). We tag the page as the mobile shell so the Speedrun screens
// navigate client-side (SvelteKit routing) rather than via Qt window states,
// which lets Playwright follow the flow: decks -> deck overview -> study.

import { expect, test } from "./fixtures";

const DEMO_DECK = "MCAT Biochemistry (demo)";

test.beforeEach(async ({ page }) => {
    // Drive the screens through client-side routing (the AnkiDroid shell path).
    await page.addInitScript(() => {
        (window as unknown as { speedrunPlatform?: string }).speedrunPlatform = "mobile";
    });
});

test("authored study flow: decks -> overview -> review", async ({ page }) => {
    // The decks home lists the seeded authored deck (SpeedrunListDecks RPC).
    await page.goto("/speedrun-decks");
    const deck = page.getByRole("button", { name: DEMO_DECK });
    await expect(deck).toBeVisible();

    // Opening the deck routes to its study overview, which reads the authored
    // engine: the three scores and the concept tree built from the authored
    // hierarchy + per-concept study progress.
    await deck.click();
    await page.waitForURL(/speedrun-study\/\d+/);
    await expect(page.getByRole("heading", { name: "Concept tree" })).toBeVisible();
    // The three score rings, by their exact labels (the abstain notes also
    // mention the score names, so match exactly).
    await expect(page.getByText("Memory", { exact: true })).toBeVisible();
    await expect(page.getByText("Performance", { exact: true })).toBeVisible();
    await expect(page.getByText("Readiness", { exact: true })).toBeVisible();
    // The concept tree shows the authored subjects (built from the hierarchy).
    await expect(page.getByText("Enzymes").first()).toBeVisible();

    // Starting study opens the bespoke review screen for the same deck.
    await page.getByRole("button", { name: "Start" }).click();
    await page.waitForURL(/speedrun-review\/\d+/);
    await expect(page.locator("body")).toBeAttached();
});
