# Speedrun: Decision Log

> The alternatives ledger. Every meaningful choice in the [PRD](prd-speedrun.md) and specs traces to an entry here. IDs are stable and never renumbered; superseding choices get a new entry, not an edit. Shape is fixed: **Chose → Considered (why rejected) → Gaps / risks.**
>
> Scope: these are the *initial* decisions seeding the Wednesday milestone of Speedrun, with Friday/Sunday context where a Wednesday choice depends on it.

## Index

| ID | Title | Status |
| :-- | :-- | :-- |
| [D1](#d1) | Exam = MCAT | resolved |
| [D2](#d2) | Two-section study model (blocked → interleaved) | resolved |
| [D3](#d3) | Rust change = topic-grouped (blocked) queue | resolved |
| [D4](#d4) | Blocked→interleaved transition is mastery-gated | resolved |
| [D5](#d5) | Principle-first scaffold + static feedback map (no AI) | resolved |
| [D6](#d6) | Concept teaching = contrasting cases, then tell | resolved |
| [D7](#d7) | Three scores shown separately, never blended | resolved |
| [D8](#d8) | Memory score = aggregated FSRS retrievability + range | resolved |
| [D9](#d9) | Give-up rule threshold | resolved |
| [D10](#d10) | Performance + Readiness designed-but-deferred | resolved |
| [D11](#d11) | Topic taxonomy = AAMC content outline via tags | resolved |
| [D12](#d12) | Coverage % computed from taxonomy vs deck | resolved |
| [D13](#d13) | Mobile = fork AnkiDroid | resolved |
| [D14](#d14) | Sync deferred to Friday | resolved |
| [D15](#d15) | Docs home = `docs/plan/` | resolved |
| [D16](#d16) | Blocked-queue ordering = weakness × topic weight | resolved |
| [D17](#d17) | Expose the queue via a new protobuf message | resolved |
| [D18](#d18) | MCAT deck/content source | open |
| [D19](#d19) | App-teacher UI = note types + card-template JS | resolved |
| [D20](#d20) | User-facing mode labels = Learn / Practice | resolved |
| [D21](#d21) | Contrasting-case sourcing = authored seed, then AI | resolved |
| [D22](#d22) | Taxonomy seed = inline Rust, leaf-only weights | resolved |
| [D23](#d23) | Weakness proxy + card→topic mapping (Wednesday) | resolved |
| [D24](#d24) | Memory score concretization (range, confidence, shared signals) | resolved |
| [D25](#d25) | Defer AttemptLog table to Phase 3 / Friday | resolved |
| [D26](#d26) | Learn entry = reviewer queue-swap to topic-grouped queue | resolved |
| [D27](#d27) | Application-card Show-Answer gate (fail-open, at _showAnswer) | resolved |
| [D28](#d28) | Preload the MCAT seed deck on collection load | resolved |

---

<a id="d1"></a>
### D1: Exam = MCAT

- **Status:** resolved
- **Chose:** Build the whole product against the MCAT (472–528; four sections 118–132).
- **Considered:** LSAT (almost no facts to memorize, a flashcard-based score model is genuinely weak there); GMAT Focus (adaptive scoring adds modeling load); USMLE Step 1 (pass/fail only since 2022, no number to project). All rejected as worse fits for a memory-first, coverage-heavy app.
- **Gaps / risks:** MCAT's huge fact base makes *coverage* the hard part; a deck that skips a high-weight section can look "ready", mitigated by the coverage gate ([D12](#d12)).

<a id="d2"></a>
### D2: Two-section study model (blocked → interleaved)

- **Status:** resolved
- **Chose:** Two distinct study modes, **Learn** (topic-grouped / blocked) and **Practice** (regular spaced repetition / interleaved), and make that split the product's pedagogical spine.
- **Considered:** A single interleaved SR flow (Anki's default, simplest, but teaches no application and discards the blocked-learning benefit); a single blocked flow (no discrimination training, weak long-term retention).
- **Gaps / risks:** Two modes is more UI and more state; learners may skip Learn and lose the scaffold, mitigated by routing new topics through Learn first.

<a id="d3"></a>
### D3: Rust change = topic-grouped (blocked) queue

- **Status:** resolved
- **Chose:** The required engine change ([source §7a](../../Speedrun_%20A%20Desktop%20+%20Mobile%20Study%20App%20Built%20on%20Anki.md)) is a new **blocked-by-topic queue builder**: a review order that serves due cards grouped by AAMC topic, ordered by weakness × weight, as a mode distinct from regular SR.
- **Considered:** *Topic-aware scheduling* (mutating FSRS intervals to pull weak topics back, rejected for Wednesday: touches the scheduler, undo, and corruption guarantees, and is the worst upstream-merge risk); *mastery query only* (a read path, too thin as the headline engine change, kept as a supporting query); *points-at-stake queue alone* (good, but folded into the blocked mode's ordering, see [D16](#d16)).
- **Gaps / risks:** A new queue path must coexist with the standard scheduler without violating FSRS interval validity or undo; needs careful integration tests. Grouping changes due-card *order*, not *timing*, which keeps it safe.

<a id="d4"></a>
### D4: Blocked→interleaved transition is mastery-gated

- **Status:** resolved
- **Chose:** A topic graduates from Learn (blocked) to Practice (interleaved) when a per-topic understanding signal is met, not after a fixed count or time.
- **Considered:** Fixed-count switch (e.g., "after N cards", simple but ignores whether the learner actually understands); always-interleaved (rejected, see grounding below).
- **Gaps / risks:** Defining the understanding signal is itself a tuning problem; Wednesday uses a simple proxy (per-topic recent accuracy + FSRS stability over a threshold) and flags it for refinement. **Grounding:** interleaving introduced before a topic is understood is an "undesirable difficulty" that backfires, especially for shaky material (Hwang 2024, *Language Learning* doi:10.1111/lang.12659; "No Simple Solutions" 2024, PMC10950551, the optimal switch point "depends on what each learner has been able to understand"; Kaminske et al. 2020).

<a id="d5"></a>
### D5: Principle-first scaffold + static feedback map (no AI)

- **Status:** resolved
- **Chose:** Before an application card reveals its solution, the learner drills a **hierarchy of choices** mirroring the taxonomy, Foundation → Content Category → Topic (i.e. principle → concept → procedure, the Dufresne/Mestre HAT). A **static, authored feedback map** gives discriminating-cue feedback on a wrong choice at any level; only a complete correct path unlocks the solution. No model calls, satisfies Wednesday's no-AI rule.
- **Considered:** A single flat principle pick (rejected, it skips the level-by-level discrimination the HAT relies on); AI free-response grading of the analysis (rejected for Wednesday, no AI; deferred to Friday); no feedback, just record the choice (rejected, the literature says the scaffold *needs* feedback to improve problem solving).
- **Gaps / risks:** Authoring the feedback map is manual content work and only covers authored items; AI generalization is the Friday upgrade. **Grounding:** Dufresne, Gerace, Hardiman & Mestre 1992 (JLS 2(3):307–331) shifts novices to deep-structure categorization; the ERIC fulltext (ED310931) notes the scaffold needed *feedback and coaching* to convert into problem-solving gains, which the feedback map supplies.

<a id="d6"></a>
### D6: Concept teaching = contrasting cases, then tell

- **Status:** resolved
- **Chose:** Teach a concept by presenting two contrasting worked cases first, prompting the learner to articulate the **shared underlying concept** (what the cases have in common), *then* stating it.
- **Considered:** Asking what *differs* between the cases (rejected, the schema forms from describing the analogs' similarity, not their contrasts, per Gick & Holyoak); tell-then-practice (the default; weaker transfer, telling first blunts the benefit of the cases); single worked example (a single example transfers poorly).
- **Gaps / risks:** Requires authored contrasting pairs per concept; quality varies with authoring. **Grounding:** Gick & Holyoak 1983 (two contrasting analogs → induced schema → transfer; one example fails); Schwartz, Chase, Oppezzo & Chin 2011 (inventing-with-contrasting-cases before telling beats tell-first); Steenhof et al. 2019 (productive failure in clinical learning).

<a id="d7"></a>
### D7: Three scores shown separately, never blended

- **Status:** resolved
- **Chose:** Memory, Performance, and Readiness are always displayed as three separate scores, each carrying the full evidence envelope ([PRD §3](prd-speedrun.md#3-the-honesty-contract-the-spine-of-the-product)); no combined "overall readiness" number exists.
- **Considered:** A single headline readiness % (what users want, what the brief calls an automatic fail); blending memory into readiness (hides the bridge the project is graded on).
- **Gaps / risks:** Three numbers + ranges is more UI and more explaining; users may still anchor on readiness, mitigated by always pairing it with coverage % and the give-up rule.

<a id="d8"></a>
### D8: Memory score = aggregated FSRS retrievability + range

- **Status:** resolved
- **Chose:** Wednesday's Memory score aggregates per-card FSRS retrievability over the in-scope cards into a point estimate, with a distribution-based range (e.g., bootstrap or interval over the per-card probabilities).
- **Considered:** Training a custom memory model now (premature, calibration proof is Sunday; FSRS is already validated); a single mean with no range (violates the evidence envelope).
- **Gaps / risks:** FSRS retrievability is a per-card recall probability, not a calibrated *aggregate*; calibration is proven Sunday on held-out reviews (Brier/log-loss). The range reflects spread, not yet model uncertainty.

<a id="d9"></a>
### D9: Give-up rule threshold

- **Status:** resolved
- **Chose:** Abstain from any score unless **≥200 graded reviews in scope AND ≥50% coverage of in-scope topics**; below the line, state which condition failed and what clears it.
- **Considered:** No abstention (an automatic fail per the brief); coverage-only or reviews-only (each lets one failure mode through, e.g., 10k reviews on 20% of topics).
- **Gaps / risks:** Thresholds are a guess pending data; exposed as tunable constants and revisited once real study histories exist.

<a id="d10"></a>
### D10: Performance + Readiness designed-but-deferred

- **Status:** resolved
- **Chose:** Specify Performance and Readiness now (data model, inputs, display) but ship only Memory on Wednesday; the other two tiles abstain with a reason until Friday/Sunday.
- **Considered:** Building all three scores by Wednesday (rejected, Performance needs exam-style items + the paraphrase test, Readiness needs a score mapping, neither honest in the no-AI window); cutting them from the design entirely (rejected, Wednesday data model must not foreclose them).
- **Gaps / risks:** The Wednesday schema must already carry per-attempt timing and item provenance so Friday doesn't require a migration.

<a id="d11"></a>
### D11: Topic taxonomy = AAMC content outline via tags

- **Status:** resolved
- **Chose:** Topics are the AAMC MCAT content outline (foundational concepts / content categories); deck cards map to topic nodes via Anki tags; per-topic exam weights are stored alongside.
- **Considered:** Deck-shipped tags only (faster but inconsistent across decks, no official weights, weak coverage %); a hand-rolled taxonomy (reinvents the official outline the exam is actually built on).
- **Gaps / risks:** Tag→topic mapping is imperfect for messy community decks; needs a reviewed mapping table and an "untagged/unmapped" bucket surfaced honestly.

<a id="d12"></a>
### D12: Coverage % computed from taxonomy vs deck

- **Status:** resolved
- **Chose:** Coverage % = share of in-scope AAMC topic nodes the deck actually has mapped cards for; it gates the give-up rule and is shown on every score.
- **Considered:** Card-count coverage (a deck with 10k cards on 20% of topics would look complete, exactly the failure mode the brief calls out).
- **Gaps / risks:** Topic granularity drives the number; the same deck reads differently at concept vs category level, granularity is fixed in [`spec-topic-taxonomy`](spec-topic-taxonomy.md).

<a id="d13"></a>
### D13: Mobile = fork AnkiDroid

- **Status:** resolved
- **Chose:** The phone app is a fork of **AnkiDroid**, rebuilt against the modified Rust backend (`rsdroid`), so the engine change ships to the phone for free.
- **Considered:** iOS via Rust FFI (no turnkey AGPL reference client, Xcode/signing overhead, riskiest for a week); a thin custom Android app over the backend (max control, far more plumbing).
- **Gaps / risks:** Rebuilding `rsdroid` against a forked backend is non-trivial toolchain work and is the single most likely Wednesday blocker, front-load it. iOS is out of scope this phase.

<a id="d14"></a>
### D14: Sync deferred to Friday

- **Status:** resolved
- **Chose:** Wednesday delivers shared-engine *review* on both apps (same deck, same engine); two-way + offline sync and conflict resolution land Friday.
- **Considered:** Building sync by Wednesday (rejected, the brief explicitly says Wednesday needs reviewing the same deck, not two-way sync; sync is where data-loss bugs live and needs its own test pass).
- **Gaps / risks:** Choosing AnkiWeb sync vs a custom server is a Friday decision; the Wednesday data model must not assume single-device-only state.

<a id="d15"></a>
### D15: Docs home = `docs/plan/`

- **Status:** resolved
- **Chose:** Planning docs live in `docs/plan/` (Anki's `docs/` is otherwise build/engineering docs with no PRD/spec/ADR pattern).
- **Considered:** Top-level `*.md` (clutters the repo root); reusing `docs/` directly (mixes product planning with build docs).
- **Gaps / risks:** None material; a new convention for this fork.

<a id="d16"></a>
### D16: Blocked-queue ordering = weakness × topic weight

- **Status:** resolved
- **Chose:** Within the Learn (blocked) queue, topic order is driven by student weakness × AAMC topic weight, so the highest-value weak topics surface first ("points-at-stake" folded into blocked mode).
- **Considered:** A separate top-level points-at-stake queue ([source §7a](../../Speedrun_%20A%20Desktop%20+%20Mobile%20Study%20App%20Built%20on%20Anki.md) option 1, redundant once blocked mode exists); plain topic order (ignores value/weakness).
- **Gaps / risks:** "Weakness" needs a definition pre-Performance-model; Wednesday uses recent per-topic accuracy + FSRS stability as the proxy.

<a id="d17"></a>
### D17: Expose the queue via a new protobuf message

- **Status:** resolved
- **Chose:** The new queue is exposed through a new protobuf message in `proto/`, surfaced as a snake_case backend method and called from Python (and inherited by the phone backend).
- **Considered:** A Python-only implementation (fails the "real Rust change" requirement, 50% grade cap); reusing an existing scheduler message (muddies the new code path and its tests).
- **Gaps / risks:** Proto changes require a full build (`just check`) before downstream code sees them; sequence the proto edit first.

<a id="d18"></a>
### D18: MCAT deck/content source

- **Status:** open
- **Chose:** *(assumed)* Use a redistributable MCAT deck (or an authored subset) as "the exam deck," pre-tagged or mappable to AAMC topics, with licensing compatible with an AGPL fork.
- **Considered:** A large popular community deck (richest coverage, but redistribution/licensing is unclear) vs a small authored deck (clean license, thin coverage, risks failing the coverage gate it's meant to demonstrate).
- **Gaps / risks:** Licensing must be confirmed before redistribution; content quality (opposite-fact cards, duplicates) feeds the authoring quality bar in [`spec-study-model`](spec-study-model.md) §10. **Owner to confirm before build.**

<a id="d19"></a>
### D19: App-teacher UI = note types + card-template JS

- **Status:** resolved
- **Chose:** The contrasting-cases and scaffold interactions ship as two custom Anki **note types** (`SpeedrunConcept`, `SpeedrunApplication`) with HTML/CSS/JS card templates, plus a thin desktop hook in `aqt/reviewer.py` for Show-Answer gating and structured pick-logging. Content lives in notes, so it syncs, schedules, flows through the blocked queue, and renders in both the desktop and AnkiDroid webviews with no per-platform code.
- **Considered:** A bespoke Svelte reviewer page in `ts/` (full control and easy structured logging, but it does not render in AnkiDroid's reviewer, so it breaks the cross-platform "one experience" property and duplicates the review loop); pushing the interaction into Rust (wrong layer, this is presentation); a Python add-on with no note types (not deck-portable, does not sync).
- **Gaps / risks:** card-template JS cannot cleanly write structured logs on AnkiDroid for Wednesday, so pick-logging is desktop-first via `pycmd` and mobile logging is deferred (revlog grades still captured). The Show-Answer gate depends on template JS plus the desktop hook, and must fail open so a template bug never traps the learner on a card.

<a id="d20"></a>
### D20: User-facing mode labels = "Learn" and "Practice"

- **Status:** resolved
- **Chose:** Label the two modes with plain user verbs: **Learn** (the blocked, topic-grouped, scaffolded mode) and **Practice** (the interleaved spaced-repetition mode). Mechanical terms (blocked / interleaved) stay in the specs, not the UI.
- **Considered:** "Focused" / "Review" (rejected, "Focused" names the tool's behaviour not the user's intent, and "Review" collides with Anki's generic term for answering due cards); "Study" / "Drill" (vaguer, less honest about what each does).
- **Gaps / risks:** "Practice" slightly overlaps the everyday sense of practising problems; accepted, because the alternative ("Review") is already overloaded in Anki.

<a id="d21"></a>
### D21: Contrasting-case sourcing = authored/curated seed, then AI

- **Status:** resolved
- **Chose:** For Wednesday (no AI), the two cases per concept are **hand-authored or curated into `SpeedrunConcept` notes** for a small, complete seed topic set, drawing raw scenarios from open-licensed sources (e.g. OpenStax, CC BY) to cut authoring time, held to the [`spec-study-model`](spec-study-model.md) §10 quality bar. Friday adds AI generation of candidate pairs from a named source, gated by the gold-set checker.
- **Considered:** Auto-pairing existing deck cards (rejected, flashcards are not contrasting cases with shared deep structure); AI generation on Wednesday (rejected, violates the no-AI rule); a large authored set (rejected for the timeline, the seed is intentionally small).
- **Gaps / risks:** authoring is manual and slow, so coverage is thin until Friday's AI path (the cold-start risk in [`spec-study-model`](spec-study-model.md) §9); curated third-party scenarios must keep license compatibility ([D18](#d18)).

<a id="d22"></a>
### D22: Taxonomy seed = inline Rust data, leaf-only weights

- **Status:** resolved
- **Chose:** Encode the seed taxonomy inline in Rust (`rslib/src/speedrun/taxonomy.rs`), carry `exam_weight` only on leaf topics (structural nodes get `0.0` / `in_scope=false`), and give `coverage_pct`/`weighted_coverage` set semantics (dedupe inputs, ignore out-of-scope ids). Landed in Phase 1A (`524a8501d`).
- **Considered:** a JSON seed file (forces `Result`/`expect`, breaks the pure `Vec` contract the queue consumes); weights on every level (risks double-counting a branch in `weighted_coverage`).
- **Gaps / risks:** seed `exam_weight`s are placeholder proportions, not a specific AAMC table, and biomolecules-only; the section-mapping layer ([`spec-topic-taxonomy`](spec-topic-taxonomy.md) §4) is deferred. Both are intended future extensions; weights stay data-tunable.

<a id="d23"></a>
### D23: Weakness proxy + card→topic mapping (Wednesday)

- **Status:** resolved (concretizes [D16](#d16))
- **Chose:** `block_priority = topic_weakness(recent_accuracy, mean_retrievability) × exam_weight`. `recent_accuracy` = pass rate (button ≥ Hard) over the topic's last 50 graded revlog entries; `mean_retrievability` = mean FSRS current retrievability over the topic's due cards, with a 0.9 prior for cards lacking memory state; no-graded-history falls back to the memory signal. Within a block, `card_priority = 1 − retrievability` (weakest first). **Card→topic** = the note tag exactly equal to a `seed_taxonomy()` leaf id (smallest id wins on ties). Landed in Phase 2a (`rslib/src/scheduler/queue/topic_grouped.rs`).
- **Considered:** accuracy-only or stability-only weakness (each ignores half the signal); fuzzy/substring tag matching (ambiguous); the spec's `manual` override table (deferred per [`spec-topic-taxonomy`](spec-topic-taxonomy.md)).
- **Gaps / risks:** the 50-review window, the 0.9 prior, and the fallback are untuned heuristics; swap to the Performance-model signal in Friday/Sunday. One `get_note` per due card is O(n) DB reads, not yet benchmarked against the PRD p95<100ms target (see [B009] note on perf; tracked for Phase 2b/3).

<a id="d24"></a>
### D24: Memory score concretization (range, confidence, shared card signals)

- **Status:** resolved (concretizes [D8](#d8), [D9](#d9))
- **Chose:** `estimate` = mean FSRS current retrievability over reviewed in-scope cards (0.9 prior for no-memory cards). `range` = 95% CI of the mean (mean ± 1.96·SE), clamped [0,1]. `confidence` bands: high ≥1000 reviews & ≥0.80 coverage; medium ≥500 & ≥0.65; else low (named consts). Give-up: `MIN_GRADED_REVIEWS=200`, `MIN_COVERAGE_PCT=0.50`; `abstained` zeroes the number and names the failed condition. Card→topic + retrievability logic extracted to `rslib/src/speedrun/card_signals.rs` so the queue (2a) and score (2b) treat a card identically. Landed in 2b (`4bad223cc`).
- **Considered:** a bootstrap interval (heavier, similar at this scale); calibration-based uncertainty (deferred to Sunday per [D8](#d8), so the CI is spread, not model uncertainty); duplicating retrievability/mapping per module (drift risk, hence the shared helper).
- **Gaps / risks:** thresholds (200 / 0.50 / confidence bands) untuned ([D9](#d9) gap); the range tightens with n (honest spread, not calibration); per-card `get_note` + `get_revlog` is O(n) ([B013] perf).

<a id="d25"></a>
### D25: Defer AttemptLog table to Phase 3 / Friday

- **Status:** resolved (overrides [`spec-scores`](spec-scores.md) §7/§9.5 "from Wednesday")
- **Chose:** do not add an `AttemptLog` DB table for Wednesday. The Memory score uses existing FSRS/revlog data (which already carries per-review timing + correctness); scaffold-pick logging arrives with Phase 3's reviewer hook; the dedicated `AttemptLog` table lands when the Performance model (Friday) needs it.
- **Considered:** building `AttemptLog` now per spec §7 (avoids a Friday migration, but adds a schema migration before any consumer exists, which is premature infrastructure).
- **Gaps / risks:** Friday's Performance model will add the table + a migration; ensure revlog + the scaffold-pick logs capture timing/provenance so nothing is lost in the interim. **Overrides** spec-scores §7/§9.5 (see the override ledger in [README](README.md)).

<a id="d26"></a>
### D26: Learn entry = reviewer queue-swap to the topic-grouped queue

- **Status:** resolved
- **Chose:** "Learn" on the deck overview starts a normal review session but swaps the reviewer's next-card source to `get_topic_grouped_queue(deck_id)` (`qt/aqt/reviewer.py` + `overview.py`). Same `QueuedCards` + `answer_card` path, so FSRS scheduling and undo are unchanged; "Practice" is the unmodified normal session.
- **Considered:** a bespoke Learn screen (more code, diverges from Anki's reviewer); a new study-session type in Rust (heavier). The queue-swap reuses the proven engine path.
- **Gaps / risks:** Learn re-queries per card, so the leading block can shift mid-session (freeze the ordered id list at session start, [B013](backlog.md#b013)); Learn serves due reviews only ([B019](backlog.md#b019)).

<a id="d27"></a>
### D27: Application-card Show-Answer gate (fail-open, at `_showAnswer`)

- **Status:** resolved
- **Chose:** the scaffold Show-Answer gate lives at the reviewer's `_showAnswer` chokepoint (covers space/enter/button/auto-advance) and is **fail-open**: it probes the template's soft signals, and only an explicit "incomplete" blocks; complete/absent/unscaffolded/error/`None` all proceed. Gating is a property of the `SpeedrunApplication` card, so it applies in both Learn and Practice; non-application cards are never gated. Picks are logged with timing Python-side.
- **Considered:** template-only gating (Phase 1 soft-signal, bypassable); per-mode gating (rejected, the scaffold should gate wherever the card appears).
- **Gaps / risks:** desktop-only (pycmd seam; mobile deferred, [D19](#d19)); pick-signal format reconciliation ([B020](backlog.md#b020)).

<a id="d28"></a>
### D28: Preload the MCAT seed deck on collection load

- **Status:** resolved
- **Chose:** auto-run `add_seed_notes` on collection load (`qt/aqt/main.py` `_maybe_seed_speedrun`), idempotent and fail-open, so the **Speedrun** MCAT deck is preloaded without a manual step (the user reported there was no deck to study).
- **Considered:** a Tools-menu "Load deck" action (explicit, but not "preloaded"); guarding on deck-absence (unnecessary, `add_seed_notes` is already idempotent, verified: 2nd run adds 0).
- **Gaps / risks:** seeds 4 cards (the small authored seed; scale via Friday AI authoring); the cards are NEW, so they appear under **Practice** now and not under **Learn** until [B019](backlog.md#b019) (include new cards in the topic queue) lands; modifies the collection on first open (acceptable for the fork/demo).

---

<sub>Created with the `iris-plan` skill by Iris Cai · maintained with `iris-log`.</sub>
