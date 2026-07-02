# Speedrun: design iterations

> Dated, feedback-driven design changes over time. Append-only. Each entry has a status + category and cross-references the decision it changed. (Pre-build planning choices live in [decisions.md](decisions.md); this log captures changes prompted by feedback and, later, by testing.)

---

### 2026-07-01: Bespoke Study screen — bar-less, rings, concept tree, subject modal

- **Category:** IA + visual identity · **Status:** applied · **Prompted by:** the bespoke-study-screen plan (replace the native deck overview with a full-window SvelteKit study screen). Requirements [ST1–ST6](requirements.md).
- **Why:** the deck overview was still native HTML inside `mw.web` with the top and bottom bars, the last stock-Anki surface in the study flow. The scores also lived in a separate dashboard window reached from a bottom-bar action. Folding both into one bar-less screen makes the study entry read as its own product and puts the three scores where the learner decides whether to start.
- **Change:** a new route `ts/routes/speedrun-study/[deckId]/` rendered into `speedrunWeb` with all chrome hidden (Phase-1 Qt wiring), on the "score instrument" identity (`$lib/sass/speedrun-tokens`).
  - **Today's Progress + Start:** a single horizontal completion bar (studied vs remaining today) with the count **left today** as the headline number, one marigold **Start** (`speedrunStartStudy`), a quiet "Decks" back link, and an overflow **menu** that preserves every old bottom-bar action (Deck options, Custom study, Unbury, Edit description, and Rebuild/Empty for filtered decks) via `speedrunOverviewAction`. Because the Qt overview no longer routes to the native congrats page, the screen renders its own "all done today" state when nothing is due.
  - **Three score rings:** a circular `ScoreRing` gauge each for Memory, Performance, Readiness. The honesty contract carries over from the dashboard gauge: an abstaining or errored score draws an **empty dashed ring** with its give-up reason, never a fake fill. Readiness fills by position on the real **472–528** scale with a whole-number readout.
  - **Centered concept tree:** `ConceptTree` + recursive `ConceptTreeNode`, built by `buildConceptTree(hierarchy, progress)` — the deck's **authored** hierarchy gives the structure; the taxonomy mastery **stage** is overlaid on leaves whose title matches a tracked topic (case-insensitive), unmatched leaves read "not started", and each branch shows the mean of its descendant stage fractions. When a deck has no authored children (the seed Biomolecules deck), the tree is rebuilt from the taxonomy `progress` paths so it still shows real structure. Leaf labels reuse `MASTERY_STAGES` (the internal `hierarchy` state shows as **Applying**).
  - **More details modal:** `StatsModal` reuses the dashboard `ScoreTile` gauges for the three scores, then a **per-subject** section (grouping the new `getSpeedrunScoreBreakdown` by content-category) with meters for coverage, mean retrievability, and application accuracy, plus a short "what feeds each score" note. Esc-to-close, focus-trapped, `prefers-reduced-motion` respected.
- **Verification:** `buildConceptTree`/`buildSubjectBreakdown` unit-tested (`lib.test.ts`: authored + stage overlay + fallback + parent aggregation + subject rollup). `check:svelte`/`check:typescript`/`check:eslint`/`check:vitest` and dprint green on touched files; `just rebuild-web` built the route; screens rendered headless (light + dark) to `out/uiqa/study-*.png`. Remaining `just check` failures are the known pre-existing/environmental ones (untouched format drift, contributors minilint, installer pytest sandbox `PermissionError`).
- **Ref:** `ts/routes/speedrun-study/*` (`lib.ts`, `[deckId]/+page.{ts,svelte}`, `ScoreRing.svelte`, `ConceptTree.svelte`, `ConceptTreeNode.svelte`, `StatsModal.svelte`); reuses `speedrun-dashboard/lib.ts` + `ScoreTile.svelte` and `speedrun-hierarchy/lib.ts`.

### 2026-07-01: Modern app shell — top bar, bottom bar, decks/home rebuilt

- **Category:** IA + visual identity · **Status:** applied · **Prompted by:** user ("Make a decks/add screen that is modern (think quizlet)... redo all of the top bar and bottom bar to follow a modern app flow... Anki should be unrecognizable as anki" + "rethink the user flow of the menu; make sure all the things the user currently can do are present"). Requirements [A1–A7](requirements.md).
- **Why:** the U6 pass restyled the study cards + dashboard on the "score instrument" identity, but the desktop chrome (top toolbar's flat text links, the spreadsheet-style deck table, the plain bottom button row) still read as stock Anki. The shell is what makes it recognisable.
- **Change:** a new visual layer on top of the same backend actions.
  - **Top app bar** (`toolbar.py` + `toolbar.scss`): a `speedrun` wordmark with a small gauge mark (the product's signature), primary nav (Decks/Browse/Stats with a marigold active underline), one prominent marigold **Create**, an icon **Sync**, and an overflow **menu**. The `create_link` add-on hook and `#sync`/`updateSyncColor` contract are kept.
  - **User-flow rethink, nothing dropped:** **Create** opens a menu (Add cards / Create deck / Import file / Get shared); the overflow **⋯** re-adds the live `QAction`s from the native File/Edit/View/Tools/Help menus, so every native action (and add-on-added ones) stays reachable in-window on every platform (on macOS the native menu is still the global bar).
  - **Decks/home** (`deckbrowser.py` + `deckbrowser.scss`): the deck table became a calm library card — mono column labels, tabular count **chips** coloured by state, a marigold current-deck marker, subdeck indentation, and an inline **Create Deck**. The `tr.deck`+`id` / `top-level-drag-row` skeleton is kept so drag-drop reparent, shift-select, collapse, options, and studied-today all still work.
  - **Deck study** (`overview.scss`): one focused study card, three readouts + the marigold Study action.
  - **Bottom bars** (`toolbar-bottom.scss`, `reviewer-bottom.scss`): a quiet action strip for deck-list/overview; the reviewer keeps Edit/More/answer/timer, restyled — Again reads clay, the recommended grade wears the marigold ring, Show Answer is the primary.
- **Verification:** headless-Chrome screenshots of the exact emitted markup + compiled CSS, light + dark, for every surface (`out/uiqa`). `just check` functional gates green (mypy/ruff-lint/eslint/svelte/typescript/vitest/rust); touched files pass ruff + dprint. Remaining check failures are pre-existing/environmental (untouched pylib/rust/proto/svelte format, contributors minilint, installer pytest sandbox `PermissionError`).
- **Ref:** `qt/aqt/{toolbar,deckbrowser}.py`, `qt/aqt/data/web/css/{toolbar,toolbar-bottom,deckbrowser,overview,reviewer-bottom}.scss`.

### 2026-06-30: Visual overhaul — the "score instrument" identity (from scratch)

- **Category:** visual identity · **Status:** applied · **Prompted by:** user feedback ("anki looks ugly. restructure the entire way the visuals are formatted... do the visuals from scratch"). Requirement [U6](requirements.md).
- **Why:** the U4 polish (below) was structurally sound but never seen in a browser (its [B029] live-QA was open). Rendered, it read as the generic "quiet teal-on-clinical-neutral SaaS dashboard", flat white cards, one hairline accent, a floaty dotted ladder, exactly the statistical-average look the UI directive warns against. It took no real stance.
- **Change:** rebuilt every Speedrun surface from scratch on one deliberate identity, grounded in the subject (an MCAT readiness instrument):
  - **Palette:** dropped the healthcare teal for **cool paper + warm charcoal ink + one marigold signal** (`#df8f0a` / dark `#f2a93b`). Wrong-answers moved to a clay red so amber can mean "on track / confirmed / the payoff". One accent, spent with restraint.
  - **Type:** a **monospace readout label system** (breadcrumbs, eyebrows, stat labels, stage names, count labels) beside the reading sans, so the app reads like a measurement instrument, not a form. Big **tabular numerals** are the heroes.
  - **Signature:** the honesty contract is _drawn_. Each delivered score renders a **measurement gauge**, a track with the likely-range band and a point-estimate marker; Readiness sits on the real **472–528 scale**. The mastery view became a proportional **distribution bar + legend + bold per-topic meters**, replacing the dotted ladder.
  - **Surfaces:** study cards (`concept.css`, `application.css` + a scaffold label in `application_front.html`), the dashboard (`SpeedrunDashboard`, `ScoreTile` with the gauge, `TopicsProgress`, gauge/scale helpers in `lib.ts`), the deck study screen (`overview.scss`, amber Study action + mono count labels), and the deck list (`deckbrowser.scss`, amber current-deck marker + mono headers).
- **Verification (closes [B029]):** built a headless-Chromium screenshot harness (this machine has a browser, unlike the U4 sandbox) and captured every surface **before and after, light + dark + mobile**. `just lint` green (svelte/ts/eslint/mypy/ruff/clippy), `test-ts` (vitest) green, and the 184-check card-mode DOM suite green. Also fixed a pre-existing mypy error in `overview.py` (`_linkHandler`'s local `import aqt.speedrun_dashboard` shadowed the module `aqt`).
- **Ref:** `pylib/anki/speedrun/templates/{concept,application}.css` + `application_front.html`, `ts/routes/speedrun-dashboard/*`, `qt/aqt/data/web/css/{overview,deckbrowser}.scss`, `qt/aqt/overview.py`.

### 2026-06-30: One Speedrun visual identity across cards + menu

- **Category:** visual identity · **Status:** applied · **Prompted by:** user feedback ("no ui frontend polish"; "rework the menu to be modern / beginner-friendly").
- **Change:** replaced the browser-default serif-on-cream look (the exact AI cluster the UI directive bans) with one deliberate identity applied across every Speedrun surface: a system-first sans (renders offline on desktop + AnkiDroid), a cool clinical neutral ground, one teal product accent (`#0e7c66` / dark `#4fd1ac`) spent with restraint, and an intentional type scale. Surfaces: the study cards (concept + application), the deck study screen (labelled count cards over a teal Study action), the home deck list (one calm elevated card, tabular counts, the current deck marked in teal), and the dashboard (the topics ladder now leads and is the signature; scores follow). Each surface was verified by rendering it in headless Chromium; the deck list + dashboard were parallelized with subagents.
- **Ref:** `pylib/anki/speedrun/templates/*.css`, `qt/aqt/overview.py` + `qt/aqt/data/web/css/overview.scss`, `qt/aqt/data/web/css/deckbrowser.scss`, `ts/routes/speedrun-dashboard/*`.

### 2026-06-30: Concept prompt asks for the shared concept, not the difference

- **Category:** pedagogy · **Status:** applied · **Prompted by:** user feedback during planning.
- **Change:** the contrasting-cases card asks "what's the shared underlying concept?" rather than "what differs?". Better matches Gick & Holyoak (schema induced by articulating analog _similarity_).
- **Ref:** [D6](decisions.md#d6); [`spec-study-model.md`](spec-study-model.md) §3, §6.

### 2026-06-30: Scaffold is a hierarchy, not a single pick

- **Category:** pedagogy · **Status:** applied · **Prompted by:** user feedback during planning.
- **Change:** the principle-first step drills Foundation → Content Category → Topic (the Dufresne/Mestre HAT) instead of one flat principle choice.
- **Ref:** [D5](decisions.md#d5); [`spec-study-model.md`](spec-study-model.md) §5.

### 2026-06-30: Mode labels reframed to Learn / Practice

- **Category:** UX copy · **Status:** superseded (see below) · **Prompted by:** user feedback during planning.
- **Change:** user-facing modes renamed from Focused/Review to **Learn** / **Practice** (plain user verbs; "Review" collided with Anki's generic term). Mechanical terms (blocked/interleaved) stay in the specs.
- **Ref:** [D20](decisions.md#d20); applied across PRD + specs.

### 2026-06-30: One Study button; the progression picks the mode

- **Category:** UX · **Status:** applied · **Prompted by:** user feedback ("the learn and review buttons should be merged").
- **Change:** the two-button Learn/Practice split is gone. One **Study** button arms the state-aware topic-grouped queue, which serves blocked→mixed for Speedrun topics and falls back to the normal queue otherwise, so the learner never picks a mode. `learn`/`practice` remain as link aliases for older callers.
- **Ref:** [D30](decisions.md#d30) supersedes [D20](decisions.md#d20); `qt/aqt/overview.py`, `ftl` `studying-study`.

### 2026-06-30: One product accent across the whole Speedrun surface

- **Category:** visual identity · **Status:** applied · **Prompted by:** the U4 `frontend-design` polish pass.
- **Change:** the dashboard and topics ladder inherited Anki's default accent while the study cards used a distinct teal (`#0e7c66` / dark `#4fd1ac`). Unified everything on that one teal (a theme-aware `--sr-accent` on the dashboard root, inherited by the tiles and the ladder), so dashboard, topics, and cards read as one product. Also fixed the **Study** button losing its primary style after the button merge (the SCSS still targeted `#learn`/`#practice`, not `#study`) and renamed the dashboard window to `Speedrun: <deck>` now that it shows more than scores.
- **Note:** the changes are verified structurally (build + svelte/ts check + vitest); a live visual QA pass is still open ([B029](backlog.md#b029)) because this sandbox has no browser.
- **Ref:** `ts/routes/speedrun-dashboard/*`, `qt/aqt/data/web/css/overview.scss`, `qt/aqt/speedrun_dashboard.py`.

### 2026-06-30: Topics view = a four-stage mastery ladder

- **Category:** UX · **Status:** applied · **Prompted by:** user feedback ("a UI listing all topics and which of the 4 stages each is at").
- **Change:** the dashboard gained a Topics section. Its signature is a per-topic four-segment track filled left to right up to the topic's current rung (learning → practicing → applying → mastering), grouped under the real AAMC hierarchy, with a ladder summary counting topics on each rung. The internal `hierarchy` state is shown to the learner as **Applying** (the mechanism name isn't a stage a student recognises). One accent, spent on the track and the occupied rungs.
- **Ref:** `ts/routes/speedrun-dashboard/TopicsProgress.svelte`, `lib.ts` `buildTopicsView`; `proto` `TopicProgress.path`.

### 2026-06-30: Topic breadcrumb on the card

- **Category:** UX · **Status:** applied · **Prompted by:** user feedback ("show the topic/hierarchy when learning").
- **Change:** each Speedrun card shows its AAMC path (foundation → category → topic) as a breadcrumb. Labels come from the taxonomy itself (`topic_path_labels`), delivered by the new `GetSpeedrunCardContext` RPC and injected as `window.speedrunTopicPath` (rendered with `textContent`, so a label can never inject markup).
- **Ref:** `rslib/src/speedrun/taxonomy.rs`, `proto` `GetSpeedrunCardContext`, `qt/aqt/reviewer.py`, card templates.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
