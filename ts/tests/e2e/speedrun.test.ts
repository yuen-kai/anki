// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// End-to-end walk of the authored Speedrun study flow against a real, freshly
// seeded collection (the launcher auto-seeds the "MCAT Biochemistry (demo)"
// authored deck). We tag the page as the mobile shell so the Speedrun screens
// navigate client-side (SvelteKit routing) rather than via Qt window states,
// which lets Playwright follow the flow: decks -> deck overview -> session.

import { expect, test } from "./fixtures";

const DEMO_DECK = "MCAT Biochemistry (demo)";

test.beforeEach(async ({ page }) => {
    // Drive the screens through client-side routing (the AnkiDroid shell path).
    await page.addInitScript(() => {
        (window as unknown as { speedrunPlatform?: string }).speedrunPlatform = "mobile";
    });
});

test("decks -> study overview", async ({ page }) => {
    // The decks home lists the seeded authored deck (SpeedrunListDecks RPC): the
    // name is a heading, and each deck is an article carrying its own Study action.
    await page.goto("/speedrun-decks");
    await expect(page.getByRole("heading", { name: DEMO_DECK })).toBeVisible();
    const deckRow = page.getByRole("article").filter({ hasText: DEMO_DECK });

    // Study opens the deck's study overview, which reads the authored engine:
    // the three score dials and the concept tree built from the authored
    // hierarchy + per-concept study progress.
    await deckRow.getByRole("button", { name: "Study" }).click();
    await page.waitForURL(/speedrun-study\/\d+/);
    await expect(page.getByRole("heading", { name: "Concept tree" })).toBeVisible();
    // Each of the three score dials exposes an aria-labelled figure ("<Score>: …");
    // the score names also recur in the subject table, so target the dials.
    await expect(page.getByRole("img", { name: /^Memory\b/ })).toBeVisible();
    await expect(page.getByRole("img", { name: /^Performance\b/ })).toBeVisible();
    await expect(page.getByRole("img", { name: /^Readiness\b/ })).toBeVisible();
    // The concept tree shows the authored subjects (built from the hierarchy).
    await expect(page.getByText("Enzymes").first()).toBeVisible();
});

test("study overview -> session: intro, learn, topic-learned, grading", async ({ page }) => {
    // The full learning walk (nine concepts across eight topics, then the freshly
    // practicing cards) is long; give it room over the many study RPCs.
    test.setTimeout(180_000);

    await page.goto("/speedrun-decks");
    const deckRow = page.getByRole("article").filter({ hasText: DEMO_DECK });
    await deckRow.getByRole("button", { name: "Study" }).click();
    await page.waitForURL(/speedrun-study\/\d+/);

    // Start studying opens the bespoke session screen for the same deck.
    await page.getByRole("button", { name: "Start studying" }).click();
    await page.waitForURL(/speedrun-review\/\d+/);

    // The session is one card at a time, disclosed downward across seams; the app
    // picks what comes next (no mode picker). Drive whatever control the current
    // card shows, recording that each key moment appeared. Learning blocks come
    // first (new-topic intro -> Learn -> topic-learned), then the scheduler serves
    // the now-practicing cards for grading.
    const beginBtn = page.getByRole("button", { name: "Begin" });
    const revealBtn = page.getByRole("button", { name: "Reveal concept details" });
    const nextConceptBtn = page.getByRole("button", { name: "Next concept" });
    const checkBtn = page.getByRole("button", { name: "Check against description" });
    const goodBtn = page.getByRole("button", { name: "Good", exact: true });
    const nextCardBtn = page.getByRole("button", { name: /Next card/ });
    // Both the topic-learned and level-up celebrations advance via a full-card
    // "Continue to next card" control.
    const continueBtn = page.getByRole("button", { name: "Continue to next card" });
    const doneMsg = page.getByText("Nothing due right now");

    const anyControl = beginBtn
        .or(revealBtn)
        .or(checkBtn)
        .or(continueBtn)
        .or(doneMsg);

    let sawIntro = false;
    let sawLearn = false;
    let sawCelebration = false;
    let sawGrading = false;

    for (let i = 0; i < 80; i++) {
        // Wait for the next card's primary control, then let the card finish
        // mounting/disclosing before acting (a fresh card can re-render once as
        // its reactive body settles, detaching a just-found control mid-click).
        await anyControl.first().waitFor({ state: "visible", timeout: 20_000 });
        await page.waitForTimeout(400);

        if (await doneMsg.isVisible()) {
            break;
        }
        if (await continueBtn.isVisible()) {
            // Topic-learned (or an individual level-up): the celebration card.
            sawCelebration = true;
            await continueBtn.click();
            continue;
        }
        if (await checkBtn.isVisible()) {
            // Practice: recall, reveal the saved description, then grade. Grading
            // rides at the card's bottom and shows the next FSRS interval.
            await checkBtn.click();
            await goodBtn.click();
            await expect(page.getByText(/next review in/)).toBeVisible();
            sawGrading = true;
            await nextCardBtn.click();
            continue;
        }
        if (await revealBtn.isVisible()) {
            // Learn: two worked cases, then reveal the concept and move on. No
            // difficulty rating at this stage.
            await revealBtn.click();
            await nextConceptBtn.click();
            sawLearn = true;
            continue;
        }
        if (await beginBtn.isVisible()) {
            // New-topic intro: the path lights down the tree to the new leaf.
            sawIntro = true;
            await beginBtn.click();
            continue;
        }
        // A card this walk does not drive (e.g. a Guided locate step, only
        // reachable after multi-session progression): stop once the fresh-seed
        // moments have all been seen.
        break;
    }

    expect(sawIntro, "new-topic intro appeared").toBe(true);
    expect(sawLearn, "Learn card with Next concept appeared").toBe(true);
    expect(sawCelebration, "topic-learned celebration appeared").toBe(true);
    expect(sawGrading, "a graded card showed its next interval").toBe(true);
});
