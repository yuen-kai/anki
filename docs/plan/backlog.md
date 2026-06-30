# Speedrun: backlog

> Bugs, refactors, and open non-decision issues. One file, each tagged `type: bug | refactor | issue`. Items are closed in place (marked done), never deleted. Resolved-by-a-real-choice items graduate to [decisions.md](decisions.md).

## Index

| ID | Title | Type | Status |
| :-- | :-- | :-- | :-- |
| [B001](#b001) | rsdroid rebuild against modified backend (mobile long-pole) | issue | open |
| [B002](#b002) | oneshot verify.sh doesn't match Anki's build | issue | open |
| [B003](#b003) | Anki build does not auto-install n2/ninja | bug | known-gap |
| [B004](#b004) | Planning docs + project memory are untracked | issue | fixed |
| [B005](#b005) | Fresh worktree build needs ftl submodule init + PROTOC | issue | open |
| [B006](#b006) | pylib/tests/__init__.py eager backend import blocks pure pytest | issue | known-gap |
| [B007](#b007) | Scaffold seed references foundation ids absent from taxonomy seed | issue | open |
| [B008](#b008) | `just check` qt installer tests fail in sandbox (Briefcase cache) | issue | known-gap |
| [B009](#b009) | Scaffold template JS + note-type installer lack automated tests | refactor | open |
| [B010](#b010) | Scaffold sessionStorage keys shared across cards | bug | open |
| [B011](#b011) | `cargo build -p anki` fails standalone (use `cargo test`) | issue | known-gap |
| [B012](#b012) | Committed tree not dprint/rustfmt-clean (`just fix-fmt` rewrites files) | refactor | open |
| [B013](#b013) | Topic-queue perf unbenchmarked (O(n) get_note vs p95<100ms) | issue | open |
| [B014](#b014) | Sandbox blocks tsx IPC pipe in `just test-py` build | issue | known-gap |
| [B015](#b015) | Topic-queue undo/interval-equivalence proof untested | issue | fixed |
| [B016](#b016) | Topic queue leaves `active_decks` pointing at its deck | bug | open |
| [B017](#b017) | Memory score refinements (range semantics, scope, tests) | issue | open |
| [B018](#b018) | Markdown not dprint-clean fails `just check` (check:format) | issue | known-gap |
| [B019](#b019) | Learn shows due reviews only (new/learning excluded) | issue | open |
| [B020](#b020) | Scaffold pick-signal format reconciliation | issue | open |
| [B021](#b021) | No automated UI regression test (dashboard + Qt Learn flow) | refactor | open |

---

<a id="b001"></a>
### B001: rsdroid must be rebuilt against the modified rslib for engine changes to reach mobile

- **Type:** issue · **Status:** open · **Severity:** high
- **Discovered:** 2026-06-30 during build bring-up (base AnkiDroid packaged the upstream prebuilt `librsdroid.so`).
- **Ref:** [`spec-mobile-shared-engine.md`](spec-mobile-shared-engine.md) §6.
- **Context:** stock AnkiDroid bundles a prebuilt backend, so none of our Rust changes (e.g. `GetTopicGroupedQueue`) appear on Android until `rsdroid` is cross-compiled against our `rslib` and repackaged. This is the single highest-risk Wednesday task.
- **Next:** prove the rsdroid → AnkiDroid native rebuild path on a trivial backend change before relying on it.

<a id="b002"></a>
### B002: oneshot `verify.sh` does not verify Anki

- **Type:** issue · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30 during oneshot setup.
- **Ref:** `~/.cursor/skills/oneshot/scripts/verify.sh`; Anki `justfile`.
- **Context:** the harness runs npm scripts / pytest / ruff / mypy from the worktree root, but Anki's real gate is `just check` (Rust + TS + Python via ninja). Default verify would no-op or mislead.
- **Resolution (planned):** set `SMOKE_CMD` / verify per task to the real recipe (`cargo test -p anki <module>`, `just test-rust`, `just check`) rather than the generic harness.

<a id="b003"></a>
### B003: `just run` aborts: n2/ninja missing, not auto-installed

- **Type:** bug · **Status:** known-gap
- **Discovered:** 2026-06-30 by the build bring-up subagent.
- **Ref:** Anki build system; `tools/install-n2`.
- **Context:** a clean machine hits `n2 and ninja missing/failed. did you forget 'bash tools/install-n2'?`. Build system does not fetch it automatically.
- **Resolution:** install once (`bash tools/install-n2` or `cargo install --git https://github.com/evmar/n2.git`). Documented in [README](README.md). Not our bug to fix (upstream); just a setup prerequisite.

<a id="b004"></a>
### B004: Planning docs + project memory are untracked

- **Type:** issue · **Status:** fixed
- **Discovered:** 2026-06-30.
- **Ref:** `docs/plan/*`. Specs committed in `13f38ee`; iris-log docs committed in the foundation commit.
- **Context:** worktree-based parallel implementation needs the specs/contracts on a committed base, otherwise fresh worktrees branched from HEAD won't contain them.
- **Resolution:** specs already tracked at `13f38ee`; README/backlog/design-iterations committed on `main` as the foundation, so worktrees branched from HEAD inherit the full plan.

<a id="b005"></a>
### B005: Fresh worktree build needs ftl submodule init + PROTOC

- **Type:** issue · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30 by the Phase 1A subagent building in an isolated worktree.
- **Ref:** `rslib/i18n/gather.rs:62` (`anki_i18n` panic); `anki_proto` build.
- **Context:** a fresh git worktree has no checked-out ftl submodules, so `anki_i18n` panics; and raw `cargo` has no `protoc`, so `anki_proto` fails. The main checkout doesn't hit this because submodules + protoc are already present from prior `just` runs.
- **Workaround (bootstrap every Rust worktree):** `git submodule update --init ftl/core-repo ftl/qt-repo` (add `qt/installer/mac-template` before `just test-py`) and `export PROTOC="$(git rev-parse --show-toplevel)/out/extracted/protoc/bin/protoc"`. **Sandbox note:** worktrees live outside the sandbox's writable root, so `cargo`/`git`/`just` must run with the sandbox disabled, otherwise they silently operate on the main repo (hit in Phase 2a). Baked into the Phase 2 prompts.

<a id="b006"></a>
### B006: `pylib/tests/__init__.py` eager backend import blocks pure pytest

- **Type:** issue · **Status:** known-gap · **Severity:** low
- **Discovered:** 2026-06-30 by the Phase 1B subagent.
- **Ref:** `pylib/tests/__init__.py` (`from anki.lang import set_lang` → imports `anki._backend` / `_rsbridge`).
- **Context:** any pure pylib test fails at pytest *collection* without a full backend build, because the test package's `__init__` imports the Rust backend. Speedrun's pure tests were verified via `python3 -m unittest` instead (22/22).
- **Possible fix:** a guarded/lazy import there or a `conftest.py` so pure tests run under pytest without a build. Not our bug; low priority.

<a id="b007"></a>
### B007: Scaffold seed references foundation ids absent from the taxonomy seed

- **Type:** issue · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30 during Phase 1 integration.
- **Ref:** `pylib/anki/speedrun/seed_content.py` (level-1 distractors `mcat::cellular_systems`, `mcat::organ_systems`); `rslib/src/speedrun/taxonomy.rs` seed has only `mcat::biomolecules`.
- **Context:** the scaffold's level-1 choice deliberately adds sibling-foundation distractors so the principle pick is a real discrimination, but those ids aren't in the (biomolecules-only) taxonomy seed. Harmless now (they're label+feedback only, never looked up in the tree), but must be reconciled when the taxonomy expands.
- **Next:** when the taxonomy grows beyond biomolecules, align these ids/labels with the canonical AAMC foundation names.

<a id="b008"></a>
### B008: `just check` qt installer tests fail in the sandbox (Briefcase cache dir)

- **Type:** issue · **Status:** known-gap (environmental) · **Severity:** low
- **Discovered:** 2026-06-30, Phase 1 integration gate.
- **Ref:** `qt/tests/test_installer.py::test_compile_fails_loudly`, `::test_build_and_package`.
- **Context:** `just check` reached the qt packaging-test step with everything before it green (Rust fmt/clippy/tests, TS, pylib lint/tests), so the Phase 1 code is clean. The 2 installer tests fail only because Briefcase can't create `~/Library/Caches/org.beeware.briefcase` in the sandbox (and would need network for support packages). Pre-existing upstream tests, unrelated to Speedrun.
- **Workaround:** set `BRIEFCASE_HOME` to a writable dir and run with network / outside the sandbox for a fully-green `just check`. For feature verification, the Rust + Python + lint portion is the relevant gate and it passed.

<a id="b009"></a>
### B009: Scaffold template JS + note-type installer lack automated tests

- **Type:** refactor · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30, Phase 1 code review ([deb3bc73] Important #3 / Minor #7).
- **Ref:** `pylib/anki/speedrun/templates/application.js` (right/wrong/cue + unlock logic); `pylib/anki/speedrun/notetypes.py` (installer/seeder, needs the backend).
- **Context:** the scaffold logic that actually runs in review is reimplemented in `application.js`, but only the parallel Python copy (`feedback.py`) is tested, so the two can silently drift (spec-study-model AC8 only met for the Python path). `notetypes.py` is untested (needs the backend).
- **Next:** add a jsdom/e2e test under `ts/tests/e2e/` for the template chooser, and a backend-level test for `add_seed_notes` idempotency, when Phase 3 wires the `aqt` hook + seeding.

<a id="b010"></a>
### B010: Scaffold sessionStorage keys are shared across cards

- **Type:** bug · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30, Phase 1 code review (Minor #6).
- **Ref:** `pylib/anki/speedrun/templates/application.js` (`speedrun:app:trail`); `concept.js` (`speedrun:concept:answer`).
- **Context:** the keys are global and reused by every card; normal review resets on render, but undo/back navigation after starting another card can briefly surface a prior card's trail/answer before the reset.
- **Next:** namespace the key by card/note id when wiring the reviewer hook in Phase 3.

<a id="b011"></a>
### B011: `cargo build -p anki` fails standalone; use `cargo test`

- **Type:** issue · **Status:** known-gap · **Severity:** low
- **Discovered:** 2026-06-30, Phase 2a.
- **Context:** `cargo build -p anki` fails because the workspace's tokio features don't enable `io-util`; `cargo test -p anki` works because dev-deps (wiremock/reqwest) unify the feature in. Use `cargo test` for Rust verification, or `just` recipes for real builds.

<a id="b012"></a>
### B012: Committed tree isn't dprint/rustfmt-clean

- **Type:** refactor · **Status:** in-progress · **Severity:** low
- **Discovered:** 2026-06-30, Phase 2a; confirmed at the Phase 2 closing gate.
- **Context:** the committed tree wasn't rustfmt/dprint-clean; `just check`'s `cargo fmt --check` failed on `taxonomy.rs` (Phase 1A was hand-formatted without rustfmt).
- **Resolution (rust):** rustfmt applied to the speedrun Rust modules (`bdf52d5af`); `cargo fmt` is now clean for our code.
- **Remaining:** markdown `dprint` is split out to [B018](#b018); the user's root source docs are excluded (their inputs, not project source).

<a id="b013"></a>
### B013: Topic-grouped queue performance is unbenchmarked

- **Type:** issue · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30, Phase 2a code review.
- **Ref:** `rslib/src/scheduler/queue/topic_grouped.rs` (one `get_note` per due card).
- **Context:** per due card the queue does a `get_note` + a *full* revlog load, nothing is cached, and because ordering is global the entire due set is rescanned on every call (so `fetch_limit`/paging stays O(n) per call). The PRD targets p95<100ms on a 50k deck ([`prd-speedrun`](prd-speedrun.md) §7); the Phase 2a review judged the current design will likely miss it.
- **Next:** dedicated perf pass before the reviewer hot path / mobile: cache the ordered build per session, replace per-card `get_note` + full-revlog loads with a single join, then benchmark on the 50k deck.

<a id="b014"></a>
### B014: Sandbox blocks tsx IPC pipe (`just test-py` build step)

- **Type:** issue · **Status:** known-gap (environmental) · **Severity:** low
- **Discovered:** 2026-06-30, Phase 2a integration.
- **Ref:** `ts/tools/markpure.ts` run via `just _test-py` (tsx `createIpcServer`).
- **Context:** `just test-py` rebuilds the generated TS lib; tsx tries to `listen` on a Unix socket in `$TMPDIR` and the sandbox denies it (`EPERM`), failing the build before pytest runs. Same class as B008 (sandbox-only, not our code). The Rust/rsbridge rebuild itself succeeded first.
- **Workaround:** run `just test-py` with the sandbox disabled. `cargo test` for Rust is unaffected.

<a id="b015"></a>
### B015: Topic-queue undo / interval-equivalence proof untested

- **Type:** issue · **Status:** fixed · **Severity:** medium
- **Discovered:** 2026-06-30, Phase 2a code review.
- **Ref:** `rslib/src/scheduler/queue/topic_grouped.rs` tests; [`spec-engine-topic-queue`](spec-engine-topic-queue.md) §8-§9 (AC5/AC6); source §7a ("proof undo works and the collection does not corrupt").
- **Context:** additive safety is sound by construction (answering uses the unchanged `AnswerCard` RPC; the new RPC only returns `QueuedCards`), but no test answered a card from this queue to check undo/interval equivalence.
- **Resolution:** Phase 2b added `answering_topic_queue_card_is_undoable_and_matches_default_interval` (`topic_grouped.rs`): answers via the unchanged path, undoes (asserts card row + revlog restored), and confirms the interval matches the default queue. Green on merged `main`.

<a id="b016"></a>
### B016: Topic queue leaves `active_decks` pointing at its deck

- **Type:** bug · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30, Phase 2a code review (Minor).
- **Ref:** `rslib/src/scheduler/queue/topic_grouped.rs` (`build_queues` writes the temporary `active_decks` table); `rslib/src/storage/deck/mod.rs` (`active_decks` is a temp table).
- **Context:** the RPC's `build_queues` sets the session-local `active_decks` to its `deck_id`; readers (`col.decks.active()`, congrats, some stats) can transiently reflect the RPC's deck if Learn and Practice interleave in one session. Not persisted/synced/undoable; self-heals on the next default build.
- **Next:** when Phase 3 wires Learn/Practice switching, snapshot/restore `active_decks` around the RPC or pass the actually-studied deck.

<a id="b017"></a>
### B017: Memory score refinements (range semantics, scope, tests)

- **Type:** issue · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30, Phase 2b review (Important #1 + Minors).
- **Ref:** `rslib/src/speedrun/memory_score.rs`; `pylib/tests/test_speedrun_memory_score.py`.
- **Items:** (a) range is a 95% CI of the *mean* (tightens with n): product sign-off on whether to show per-card spread instead ([D24](decisions.md#d24)); (b) give-up is whole-deck-only (no per-topic abstention yet, though spec §5 allows per-scope); (c) the give-up boundary (exactly 200 / 0.50) is untested; (d) the Python e2e only exercises the 0.9 no-memory prior (degenerate range): add a real-memory-state case; (e) `graded_reviews` includes learning-state ratings (decide whether Memory should count review-state only).
- **Next:** address in Friday/Sunday score work; add boundary + real-memory tests.

<a id="b018"></a>
### B018: Markdown not dprint-clean fails `just check` (check:format)

- **Type:** issue · **Status:** known-gap · **Severity:** low
- **Discovered:** 2026-06-30, Phase 2 closing gate.
- **Ref:** `check:format` (dprint); root source docs `Brainlift MCAT.md` + `Speedrun_ A Desktop + Mobile Study App Built on Anki.md` (pre-existing, tracked since `13f38ee`); also `docs/plan/*.md` (hand-written, not yet dprint-canonical).
- **Context:** `just check`'s format step runs dprint over markdown; the user's root source docs (one carries a large base64 image) and our plan docs aren't dprint-canonical, so `check:format` still fails after `cargo fmt` is clean. All CODE checks pass (clippy, cross-language tests, mypy, TS, rustfmt).
- **Next:** optionally dprint `docs/plan/` in a cleanup; do NOT auto-reformat the user's root source docs (their inputs); exclude them from the check or leave as pre-existing.

<a id="b019"></a>
### B019: Learn shows due reviews only (new/learning excluded)

- **Type:** issue · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30, Phase 3b (gap G2).
- **Ref:** `rslib/src/scheduler/queue/topic_grouped.rs` (gathers Review entries only); `qt/aqt/reviewer.py` Learn mode.
- **Context:** the topic-grouped queue serves due **review** cards; new/learning cards are excluded, so freshly seeded Speedrun cards don't appear in Learn until graduated (study once via Practice first). Hurts the first-run demo.
- **Next:** decide whether Learn should include new cards (gather new + review into the topic blocks) for the Wednesday demo; if so, extend the queue gather.

<a id="b020"></a>
### B020: Scaffold pick-signal format reconciliation

- **Type:** issue · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30, Phase 3b (gap G1).
- **Ref:** `pylib/anki/speedrun/templates/application.js` (emits `speedrun:pick:<level>:<0|1>`); `qt/aqt/speedrun.py` `parse_pick_signal`; [`spec-study-model`](spec-study-model.md) §5 (specced `<level>:<node>:<ms>`).
- **Context:** the template emits a correctness-coded pick (`0|1`), not the node id + ms; the hook parses the real format and timestamps Python-side (forward-compatible). Spec, template, and hook should converge on one schema carrying the chosen node id + timing for the Friday Performance model.
- **Next:** align the template emit, hook parse, and spec on a single pick-signal schema.

<a id="b021"></a>
### B021: No automated UI regression test (dashboard + Qt Learn flow)

- **Type:** refactor · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30, Phase 3 (gaps 3a/3b G6).
- **Ref:** `ts/routes/speedrun-dashboard/*` (no committed e2e); the Qt-native Learn/reviewer flow.
- **Context:** the dashboard has screenshots but no committed e2e; the Learn/gate flow is Qt-native, so the `ts/tests/e2e` Playwright harness (mediasrv pages only) can't drive it. The 28 qt unit tests cover the gate logic, but the end-to-end UI has no regression guard.
- **Next:** add a Playwright smoke for the dashboard page (mirror `ts/tests/e2e/sanity.test.ts`); evaluate a Qt-level test for the Learn flow.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
