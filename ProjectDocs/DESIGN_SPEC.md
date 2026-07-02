# Synapse — MCAT Study App · Design Spec

Companion to `MCAT Study App.dc.html` (the visual canvas). This doc carries intent, structure, states, and rules that pixels alone don't convey. Runs on **desktop (primary) and phone**. Working name **Synapse**.

All literal content in the canvas is **placeholder** (`Deck 1`, `Group 1`, `Topic 1`, `Concept 1`, `Problem 1`, `Answer 1`). These map to author-entered content from the Builder — never ship them as copy.

---

## 1. Core mental model

**Hierarchy (author-defined depth):**
`Deck → Group…(0+ intermediate nodes)… → Topic (leaf) → Concept (card) → Problems`
- **Topic** = a leaf node. It is the only node that holds concepts.
- **Concept** = one card. Has a title, a content description (the explanation), and practice problems (each with 4 answer choices, one marked correct).
- Non-leaf nodes only aggregate; they hold no concepts directly.

**Mastery stages (fixed ladder, per concept):**
`Learn → Practice → Guided → Solo`
Every concept sits at one stage. A leaf shows its dominant/least-advanced stage; parents roll up combined progress from descendants. Rollup "pulses up the branches" so a parent's progress reads as the sum of its leaves.
- **Learn** — first acquisition. Two worked problems shown with only correct answers; learner states what they share; then the concept detail is revealed. No difficulty rating.
- **Practice** — free recall: given the concept title, recite its description, then self-check against the saved text.
- **Guided** — multiple-choice concept question **plus** a preceding step to locate the concept in the hierarchy (Group → Topic). Same question body as Solo.
- **Solo** — the same multiple-choice concept question answered straight, no locating step.

Stage glyph (used everywhere — cards, tree leaves, level-up): **4 small segments**, filled up to the current stage in that stage's color.

**Blocking → interleaving (sequencing principle):** a concept is first **learned in a block** — introduced and driven to the Learn threshold in isolation, before its neighbors compete for attention. Once blocked/learned, it is **interleaved** with other concepts in later stages (Practice/Guided/Solo) rather than drilled one in a row. So: **block to acquire, interleave to retain.** The session scheduler enforces this — new concepts are not interleaved until blocked.

**Upgrade rules:**
- Cards usually upgrade **individually** → *Level-up* screen.
- When **every** concept in a topic clears Learn, they upgrade **together** → *Topic-learned* screen.

---

## 2. Visual system

**Type**
- `IBM Plex Sans` — all UI text, headings, body.
- `IBM Plex Mono` — labels, meta, counts, numerals, tags (UPPERCASE, letter-spaced ~.12–.16em).
- Optional `Source Serif 4` reserved for editorial/tier numerals if needed.

**Palette**
- App background `#e9edf2`; card surface `#f5f7fa`; inset/nested surface `#edf0f5`/`#eef1f6`; white `#fff` for input/answer tiles.
- Ink `#26303c`; secondary text `#5c6879`; muted/meta `#7a8698`; faint `#94a0b0`; hairline borders `rgba(38,48,60,.08)` / `#e5eaf1`.
- **Accent (brand/primary action):** coral `#ee5d6c` (light `#f4808c`).
- Dark actions/buttons: ink `#26303c`.

**Stage colors (consistent everywhere):**
- Learn `#7a8698` (gray) · Practice `#4c7fd0` (blue) · Guided `#e4a33d` (amber) · Solo `#3fa976` (green).
- Inactive segment `#c8d4e4`/`#dde3eb`.

**Score colors:** Memory `#ee5d6c` · Performance `#4c7fd0` · Readiness `#3fa976`.

**Form**
- Radius: cards 16–20px, tiles/controls 8–12px, pills 20px.
- Card shadow: `0 1px 2px rgba(31,42,55,.05), 0 10px 28px rgba(31,42,55,.08)`.
- Density: generous. Desktop base width ~1240px. Never use dark fills for hierarchy roots — keep them light.

**Copy tone:** UI shows, never tells. No instructional/meta narration inside screens (e.g. no "opens the builder", "tap to edit"). Such notes live only in the canvas captions, not the product.

---

## 3. Screens

### 3a. Decks (home) — canvas `2a`
Entry screen. Header (brand, exam countdown, avatar) + deck rows + New deck.
- **Deck row:** name, `N topics · N concepts · last studied…`, a single **completion bar** (% past-Learn share), and exactly **two actions: Study · Details**. Details = edit/manage (opens Builder). No new/learning/due breakdown here.
- **New deck** → opens Builder (hierarchy).
- **Empty state (`2h`):** dashed create affordance, "No decks yet", one primary New-deck CTA.

### 3b. Study screen (per deck) — canvas `2b`
Opened by Study. Four regions:
1. **Today** — cards left today + progress bar + **Start studying** (≈time). The app chooses what to serve; user does not pick a mode. **Start studying is disabled whenever nothing is due.**
2. **Three score dials — Memory / Performance / Readiness, never blended.** Each dial shows: a **likely range** (not a point), **coverage** (% of deck / exam skills / exam content it covers so far), and **reasons** (short evidence lines). Each is independent.
   - **Insufficient-data state (required):** show **no number** ("—", dashed dial) + **what's missing** (e.g. "Missing: 3 scored timed sessions · ≥50% coverage"). Readiness especially is **never inferred from memory alone** — it waits for scored, timed evidence. Canvas shows Readiness in this empty state.
3. **Concept tree** — centered, visual (root → groups → leaves), each leaf tagged with its stage (4-seg glyph + color); parents show combined progress. Legend maps the four stage colors.
4. **More details** — per-subject table: Memory / Performance / Coverage per subject, explaining how each score is built (weighted by exam-blueprint share). Bottleneck subject highlighted.
- **Options menu (⚙, header):** opens Anki options (deck options, custom study). Present on both this screen and the not-started edge (`2i`).
- **Deck-not-started edge (`2i`):** Today = "Nothing due yet" with **Start studying disabled**; all three scores locked at "—" with their missing-data reasons; options menu still available.

### 3c. Study session — canvas `2c` (desktop, landscape cards) + `2g` (phone)
The app decides the next card; **no mode picker**. Each moment is **one screen with progressive disclosure** — later steps reveal **below** on the *same* screen (dashed "▼ … same screen" divider marks the seam), never a new page.

- **New-topic intro:** announces the topic, then animates *down the concept tree* along the highlighted path to the new leaf (only the path to the new leaf highlights). Leaf marked NEW.
- **Learning (Learn):** stage tag + "TOPIC · N of M concepts left" indicator at top. Two problems shown with only their correct answers → learner types the shared idea → concept detail reveals below. Primary button: **Next concept**. No difficulty rating.
- **Practice:** concept title → learner recalls the description → saved description drops in below with a match indicator (self-check). Then grading section (below).
- **Guided:** Step ① locate concept in hierarchy (Group→Topic chips) → Step ② the MCQ → checked result. Then grading section.
- **Solo:** Step ① the MCQ (same body as Guided, no locate step) → checked result. Then grading section.
- **Wrong pick (Guided/Solo) (`2j`):** a wrong choice shows an inline correction (why it's wrong) before advancing; correct answer then confirmed.
- **Grading — not a separate screen.** It rides as the **bottom section** of graded cards (Practice, Guided, Solo — *not* Learn). "How hard was this card?" → Again / Hard / Good / Easy → shows resulting next interval + up-next. Feeds the spacing schedule.

**Level-up (individual) — in `2c`:** celebratory screen. 4-node stepper (Learn→Practice→Guided→Solo) with completed nodes filled in their colors, the newly-reached node **igniting** (glow ring + medal + restrained confetti). Names the concept; one meaning line stating what the new stage entails. No XP, no level pills.

**Topic-learned (upgrade-together) — in `2c`:** shown only when all concepts in a topic clear Learn at once. Lists every concept in the topic advancing one stage in unison (each with before/after segment glyph), headed "TOPIC LEVELED UP".

### 3d. Builder — canvas `2d`
Three nested levels; **Create-hierarchy stays a list** (outline), not a visual tree.
1. **Hierarchy** — indented outline of nodes; each level is nameable inline; progress rolls up to parents. Actions: **Add topic** / **Add subtopic**. Leaf nodes (topics) open their concepts.
2. **Concepts list** — the concepts under one leaf; open a concept to edit.
3. **Concept editor** — concept **title**, **content description** (the explanation), and **practice problems**, each with **four answer choices** and the correct one marked.

---

## 4. Phone layouts — canvas `2e`–`2g`
Same information architecture, single-column, in a phone frame. `2e` Decks (stacked deck cards, Study/Details), `2f` Study (Today strip, three compact score tiles incl. Readiness "—" empty, condensed tree), `2g` Session (single card, same progressive-disclosure seam). Hit targets ≥44px. Desktop and phone share all logic.

---

## 5. Interaction & motion
- **Tree pulse:** progress animates up branches to the root (leaves visibly "add up" into parents).
- **New-topic intro:** dashed connectors flow downward along *only* the path to the new leaf; seamless loop.
- **Level-up:** node ignites (ring-pop + medal + short confetti); earned, not constant. Keep celebratory but restrained; matches the light clinical theme (no dark takeover screens).
- **Progressive disclosure:** reveals expand the current screen downward; preserve scroll continuity; never navigate away mid-moment.
- Respect `prefers-reduced-motion` (static fallbacks for pulses/confetti/marching dashes).