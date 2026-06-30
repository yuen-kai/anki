# Speedrun — backlog

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

---

<a id="b001"></a>
### B001 — rsdroid must be rebuilt against the modified rslib for engine changes to reach mobile

- **Type:** issue · **Status:** open · **Severity:** high
- **Discovered:** 2026-06-30 during build bring-up (base AnkiDroid packaged the upstream prebuilt `librsdroid.so`).
- **Ref:** [`spec-mobile-shared-engine.md`](spec-mobile-shared-engine.md) §6.
- **Context:** stock AnkiDroid bundles a prebuilt backend, so none of our Rust changes (e.g. `GetTopicGroupedQueue`) appear on Android until `rsdroid` is cross-compiled against our `rslib` and repackaged. This is the single highest-risk Wednesday task.
- **Next:** prove the rsdroid → AnkiDroid native rebuild path on a trivial backend change before relying on it.

<a id="b002"></a>
### B002 — oneshot `verify.sh` does not verify Anki

- **Type:** issue · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30 during oneshot setup.
- **Ref:** `~/.cursor/skills/oneshot/scripts/verify.sh`; Anki `justfile`.
- **Context:** the harness runs npm scripts / pytest / ruff / mypy from the worktree root, but Anki's real gate is `just check` (Rust + TS + Python via ninja). Default verify would no-op or mislead.
- **Resolution (planned):** set `SMOKE_CMD` / verify per task to the real recipe (`cargo test -p anki <module>`, `just test-rust`, `just check`) rather than the generic harness.

<a id="b003"></a>
### B003 — `just run` aborts: n2/ninja missing, not auto-installed

- **Type:** bug · **Status:** known-gap
- **Discovered:** 2026-06-30 by the build bring-up subagent.
- **Ref:** Anki build system; `tools/install-n2`.
- **Context:** a clean machine hits `n2 and ninja missing/failed. did you forget 'bash tools/install-n2'?`. Build system does not fetch it automatically.
- **Resolution:** install once (`bash tools/install-n2` or `cargo install --git https://github.com/evmar/n2.git`). Documented in [README](README.md). Not our bug to fix (upstream); just a setup prerequisite.

<a id="b004"></a>
### B004 — Planning docs + project memory are untracked

- **Type:** issue · **Status:** fixed
- **Discovered:** 2026-06-30.
- **Ref:** `docs/plan/*`. Specs committed in `13f38ee`; iris-log docs committed in the foundation commit.
- **Context:** worktree-based parallel implementation needs the specs/contracts on a committed base, otherwise fresh worktrees branched from HEAD won't contain them.
- **Resolution:** specs already tracked at `13f38ee`; README/backlog/design-iterations committed on `main` as the foundation, so worktrees branched from HEAD inherit the full plan.

<a id="b005"></a>
### B005 — Fresh worktree build needs ftl submodule init + PROTOC

- **Type:** issue · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30 by the Phase 1A subagent building in an isolated worktree.
- **Ref:** `rslib/i18n/gather.rs:62` (`anki_i18n` panic); `anki_proto` build.
- **Context:** a fresh git worktree has no checked-out ftl submodules, so `anki_i18n` panics; and raw `cargo` has no `protoc`, so `anki_proto` fails. The main checkout doesn't hit this because submodules + protoc are already present from prior `just` runs.
- **Workaround (bootstrap every Rust worktree):** `git submodule update --init ftl/core-repo ftl/qt-repo` and `export PROTOC="$(git rev-parse --show-toplevel)/out/extracted/protoc/bin/protoc"`. Bake into the Phase 2 subagent prompts.

<a id="b006"></a>
### B006 — `pylib/tests/__init__.py` eager backend import blocks pure pytest

- **Type:** issue · **Status:** known-gap · **Severity:** low
- **Discovered:** 2026-06-30 by the Phase 1B subagent.
- **Ref:** `pylib/tests/__init__.py` (`from anki.lang import set_lang` → imports `anki._backend` / `_rsbridge`).
- **Context:** any pure pylib test fails at pytest *collection* without a full backend build, because the test package's `__init__` imports the Rust backend. Speedrun's pure tests were verified via `python3 -m unittest` instead (22/22).
- **Possible fix:** a guarded/lazy import there or a `conftest.py` so pure tests run under pytest without a build. Not our bug; low priority.

<a id="b007"></a>
### B007 — Scaffold seed references foundation ids absent from the taxonomy seed

- **Type:** issue · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30 during Phase 1 integration.
- **Ref:** `pylib/anki/speedrun/seed_content.py` (level-1 distractors `mcat::cellular_systems`, `mcat::organ_systems`); `rslib/src/speedrun/taxonomy.rs` seed has only `mcat::biomolecules`.
- **Context:** the scaffold's level-1 choice deliberately adds sibling-foundation distractors so the principle pick is a real discrimination, but those ids aren't in the (biomolecules-only) taxonomy seed. Harmless now (they're label+feedback only, never looked up in the tree), but must be reconciled when the taxonomy expands.
- **Next:** when the taxonomy grows beyond biomolecules, align these ids/labels with the canonical AAMC foundation names.

<a id="b008"></a>
### B008 — `just check` qt installer tests fail in the sandbox (Briefcase cache dir)

- **Type:** issue · **Status:** known-gap (environmental) · **Severity:** low
- **Discovered:** 2026-06-30, Phase 1 integration gate.
- **Ref:** `qt/tests/test_installer.py::test_compile_fails_loudly`, `::test_build_and_package`.
- **Context:** `just check` reached the qt packaging-test step with everything before it green (Rust fmt/clippy/tests, TS, pylib lint/tests), so the Phase 1 code is clean. The 2 installer tests fail only because Briefcase can't create `~/Library/Caches/org.beeware.briefcase` in the sandbox (and would need network for support packages). Pre-existing upstream tests, unrelated to Speedrun.
- **Workaround:** set `BRIEFCASE_HOME` to a writable dir and run with network / outside the sandbox for a fully-green `just check`. For feature verification, the Rust + Python + lint portion is the relevant gate and it passed.

<a id="b009"></a>
### B009 — Scaffold template JS + note-type installer lack automated tests

- **Type:** refactor · **Status:** open · **Severity:** medium
- **Discovered:** 2026-06-30, Phase 1 code review ([deb3bc73] Important #3 / Minor #7).
- **Ref:** `pylib/anki/speedrun/templates/application.js` (right/wrong/cue + unlock logic); `pylib/anki/speedrun/notetypes.py` (installer/seeder, needs the backend).
- **Context:** the scaffold logic that actually runs in review is reimplemented in `application.js`, but only the parallel Python copy (`feedback.py`) is tested, so the two can silently drift (spec-study-model AC8 only met for the Python path). `notetypes.py` is untested (needs the backend).
- **Next:** add a jsdom/e2e test under `ts/tests/e2e/` for the template chooser, and a backend-level test for `add_seed_notes` idempotency, when Phase 3 wires the `aqt` hook + seeding.

<a id="b010"></a>
### B010 — Scaffold sessionStorage keys are shared across cards

- **Type:** bug · **Status:** open · **Severity:** low
- **Discovered:** 2026-06-30, Phase 1 code review (Minor #6).
- **Ref:** `pylib/anki/speedrun/templates/application.js` (`speedrun:app:trail`); `concept.js` (`speedrun:concept:answer`).
- **Context:** the keys are global and reused by every card; normal review resets on render, but undo/back navigation after starting another card can briefly surface a prior card's trail/answer before the reset.
- **Next:** namespace the key by card/note id when wiring the reviewer hook in Phase 3.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
