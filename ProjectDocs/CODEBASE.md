# Codebase map

Agent-facing reference for this repo (Speedrun, a fork of Anki). It covers how the
code is laid out, how the layers talk to each other, and where the Speedrun
additions live. This is a code map, not a product or design doc.

Last updated: 2026-07-03. One study engine (authored hierarchy); the note/taxonomy
study path is gone. Both hosts live in this repo (the Android host is under
`android/`). Landed since the last update: native self-hosted accounts + sync, AI
deck-import, the preloaded decks moved into the shared Rust engine (with a backfill),
per-concept mastery moved off config onto the cards so it merges on sync, and the
authoring editor swapped autosave for an explicit Save. The engine + desktop app are
verified building/running (green tree); the new Android host bridges (sync, AI import,
file-picker) are not yet gradle-verified — no JDK in this build env, though rsdroid
still cross-compiles for arm64. Prefer module and function names over line numbers when
navigating.

## Repo layout (the four-way split)

One monorepo, four parts. The shared engine and shared UI are not duplicated per
platform; each host embeds them.

- Backend (shared engine): `rslib/` (Rust core) with `proto/` contracts and the
  `pylib/rsbridge` / `android/bridge` language bridges into it.
- Frontend (shared UI): `ts/` (the SvelteKit app both hosts render).
- Desktop host: `qt/` (the PyQt app) + `pylib/` (the Python layer it uses to reach
  the engine).
- Android host: `android/app` (the AnkiDroid Kotlin app) + `android/bridge` (rsdroid,
  the JNI engine-bridge + `.aar` packaging, the Android analog of `pylib/rsbridge`).

There is no "android backend": the backend is the shared `rslib`. The Android side
was merged in from two former repos (`Anki-Android` -> `android/app`,
`Anki-Android-Backend` -> `android/bridge`), history dropped, no longer tied to the
upstream `ankidroid/*` remotes. `android/bridge` is excluded from the desktop Cargo
workspace; its rsdroid crate depends on the in-repo `rslib` by relative path (no
symlink). Android build outputs are gitignored and `android/**/local.properties` is
machine-local.

## Layers

Anki is multi-layer, tied together by a protobuf contract.

- `rslib/` — Rust core (crate `anki`): collection, SQLite storage, FSRS scheduler,
  search, sync, import/export, i18n, and the protobuf RPC surface. Speedrun code
  lives under `rslib/src/speedrun/`.
- `pylib/` — Python library wrapping the Rust core. `pylib/rsbridge/` is the PyO3
  bridge (compiled `_rsbridge`). `pylib/anki/` is the API. Speedrun keeps almost no
  Python now (the engine is in Rust): `pylib/anki/speedrun/` is a docstring-only
  package, and accounts reuse `collection.py`'s existing `sync_login`.
- `qt/aqt/` — PyQt6 desktop GUI. Embeds Svelte/TS pages in webviews via a local
  server (`mediasrv.py`) and bridges JS and Python with `pycmd`.
- `ts/` — Svelte/TypeScript frontend. SvelteKit routes in `ts/routes/`, shared code
  in `ts/lib/`, styles in `ts/lib/sass/`. Speedrun routes are `ts/routes/speedrun-*`.
- `proto/anki/*.proto` — service and message contracts. Codegen produces Rust
  dispatch, Python `_backend_generated.py`, and the TS `@generated/backend` module.
- `ftl/` — Fluent translations; codegen (`rslib/i18n`) gives type-safe APIs for
  Rust, Python, and TS. Edit `ftl/core` (or `ftl/qt` for Qt-only strings).
- `build/` + `justfile` — build system. Use `just` recipes only (`just --list`); do
  not call `./ninja`, `./run`, or `tools/*` directly. Generated output lands in `out/`.
- `android/` — the Android host (see "Repo layout" above): `android/app` (the
  AnkiDroid app) + `android/bridge` (rsdroid, the JNI engine bridge + `.aar`).

## RPC and data flow

Define an RPC on a `Service` in `proto/anki/<domain>.proto`, then a build
regenerates bindings. At runtime:

- Python/Qt: `col._backend.<method>(...)` -> `_backend.py` `_run_command(service,
  method, bytes)` -> `pylib/rsbridge` -> `Backend::run_service_method` (rslib) ->
  the `Collection` trait impl in `rslib/src/<domain>/service/` -> storage.
- Web (Svelte): a generated wrapper (e.g. `getMemoryScore`) -> `postProto()` POSTs
  to `/_anki/<method>` (`ts/lib/generated/post.ts`) -> `qt/aqt/mediasrv.py` routes
  it to a Python handler (`post_handler_list`) or proxies to Rust
  (`exposed_backend_list`) -> the same rsbridge path.
- Collection-scoped RPCs run under `Backend.with_col` (mutex). Mutations wrap
  `Collection::transact` (undoable).
- Direct SQL (AnkiDroid and legacy) goes through `DBProxy` -> the `db_command`
  JSON protocol.

Dispatch and codegen live in `rslib/proto_gen/src/lib.rs` (service/method indices),
`rslib/rust_interface.rs` (the `run_service_method` match), `rslib/src/services.rs`,
and `rslib/src/backend/mod.rs`.

## Build and dev

- `just run` builds pylib + qt and launches. `just check` formats, builds, and runs
  checks/tests (the final gate). `just web-watch` live-reloads the frontend.
- Quick loops: `cargo check`, `just lint`, `just test-rust|test-py|test-ts`,
  `just test-e2e`.
- Dev web pages are served at `http://localhost:40000/_anki/pages/`. SvelteKit pages
  are whitelisted in `mediasrv.py` (`is_sveltekit_page`).
- `.proto` changes need a full build to regenerate bindings.
- Generated cross-language code: `out/{pylib/anki,qt/_aqt,ts/lib/generated}`.
- Android: `android/run` is the dev loop — it **rebuilds the bridge + web assets
  first** (`cargo run -p build_rust`: ninja web artifacts -> `cargo ndk` cross-compile
  -> gradle `assembleRelease`), then `installFullDebug`. Rebuilding first is
  load-bearing: gradle treats the `.aar` as opaque, so without it TS/Rust/Svelte edits
  never reach the device (the old "my update doesn't show" bug, when `run` did a bare
  `installPlayDebug` with no rebuild). `ANDROID_SKIP_ROBOLECTRIC=1` (set by `run`) skips
  the host-test JNI for a faster loop. Needs `JAVA_HOME` (JDK 17+), `ANDROID_NDK_HOME`
  (NDK 29), and `android/{app,bridge}/local.properties` (machine-local; the app's also
  sets `local_backend=true`). The bridge bundles the shared `out/sveltekit` web assets
  into the `.aar`. Dev build is single-arch (arm64); `ALL_ARCHS=1 RELEASE=1` for a
  multi-arch release `.aar`.

## Base Anki: key files

- Collection: `rslib/src/collection/mod.rs`, transactions in `collection/transact.rs`,
  storage in `rslib/src/storage/sqlite.rs`.
- Scheduler/FSRS: `rslib/src/scheduler/` (queue builder in `queue/`, answering in
  `answering/mod.rs`, FSRS in `fsrs/`), service in `scheduler/service/mod.rs`.
- Backend: `rslib/src/backend/mod.rs`, `backend/collection.rs`.
- Python: `pylib/anki/collection.py`, `_backend.py`, `scheduler/v3.py`,
  `pylib/rsbridge/lib.rs`.
- Qt: `qt/aqt/main.py` (main window + state machine), `reviewer.py`, `deckbrowser.py`,
  `overview.py`, `webview.py`, `mediasrv.py`, `toolbar.py`, `editor.py`.
- TS: `ts/lib/generated/post.ts`, `ts/routes/+layout.*`, per-page apps under `ts/routes/`.

## Speedrun: the fork

Speedrun adds an MCAT study app. There is one study engine (decided): the
authored-hierarchy engine.

### Study engine (authored hierarchy)

- Rust: `rslib/src/speedrun/study.rs`. Per-concept mastery (`{state, seen, ratings,
  appCorrect, appTotal}`) now lives **on each concept's `SpeedrunItem` card**, in the
  card's `custom_data` (compact `sr*` keys, `ratings` packed as a digit string, window
  capped to fit the 100-byte budget). This is the anti-clobber move: config syncs
  whole-table last-writer-wins, so two offline devices editing different concepts
  clobbered each other; card `custom_data` merges per card instead. `appCorrect`/
  `appTotal` are the per-concept application-attempt counts (Performance evidence),
  written only when the concept is at Applying/Mastering. Cards are `SpeedrunItem`, one
  per authored concept, FSRS-enabled, graded via `grade_now`. Config now holds only
  `speedrun_authoring` (hierarchy); the legacy
  `speedrun_study_progress` blob is kept only for the one-time migration onto cards
  (`speedrun_migrate_progress_if_needed`, guarded by `speedrun_mastery_migrated`).
- Authoring store: config key `speedrun_authoring` (deck -> hierarchy JSON), Rust in
  `rslib/src/speedrun/authoring.rs`.
- Screen: `ts/routes/speedrun-review/` (session driver plus ConceptLearn,
  PracticeRecall, ScaffoldPicker, ProblemCard, UpgradeAnimation).
- Mastery states: Learning -> Practicing -> Applying (`hierarchy`) -> Mastering.

### Session selection (`speedrun_next_card` in `study.rs`)

`SpeedrunNextCard` reconciles the deck (materialize one `SpeedrunItem` per concept,
enable FSRS), then returns, in order:

1. `next_learning_block`: the first concept-leaf, in hierarchy tree order, that
   still holds any concept in the `learning` state, as a `learning_block`
   (`topicNodeId`, `conceptIds`, `learnedCount`, `totalCount`). `SpeedrunRecordLearned`
   marks concepts seen and flips a leaf `learning -> practicing` only once every
   concept in it is seen (topic-gated), so learning is taught one whole leaf at a time.
2. When no leaf has a learning concept, `speedrun_peek_next_card` (in
   `scheduler/queue/mod.rs`) returns the deck's next card in the stock FSRS scheduler
   order (new, then due review, then interday learning) as a `review`
   (`cardId`, `conceptId`, `state`); the concept's `state` selects the interaction
   (practicing = recall, hierarchy = scaffolded problem, mastering = unscaffolded).
3. If the scheduler has nothing due, returns `done`. There is no bespoke weakness or
   topic-grouped ordering in the review phase (that was the deleted note engine).

### Entry path (desktop)

App opens -> `deckBrowser` state -> `speedrun-decks`. Tap a deck ->
`speedrunOpenDeck` -> `overview` state -> `speedrun-study/{id}`. Study button ->
`speedrunStartStudy` (`mediasrv.py`) -> `speedrunStudy` state -> `speedrun-review/{id}`.
Mobile branches client-side via `window.speedrunPlatform`. The app-level screens
(Account, Import, Demo) open from the Tools menu on desktop and from a top nav on the
decks home on mobile (gated by `isMobileShell()`).

### Taxonomy data (kept, not an engine)

`rslib/src/speedrun/taxonomy.rs` holds the AAMC tree, labels, and exam weights (pure
data for score weighting and labels). `leaf_weight_by_label` maps a lowercased leaf
label to its exam weight; `mean_leaf_weight` is the neutral weight for an unmapped
leaf. `rslib/src/speedrun/card_signals.rs` is trimmed to `card_retrievability` (FSRS
retrievability of a card, `0.9` prior when a card has no memory state).

### Three scores (computed from the authored engine)

`rslib/src/speedrun/{memory_score,performance_score,readiness_score,scores,score_breakdown}.rs`.
All read the authored engine, so studying moves them. Weighting: a concept's weight
is its authored leaf's AAMC exam weight when the leaf title matches a taxonomy label
(case-insensitive), else `mean_leaf_weight` (a neutral share); the weighted mean
normalizes by the sum of the weights it used.

- Memory (`get_memory_score`): per concept, FSRS retrievability of its `SpeedrunItem`
  card (via `speedrun_concept_card_stats`); estimate = weight-weighted mean over
  concepts with >=1 graded review; 95% interval from the unweighted spread
  (`mean +/- Z_95*SE`). Coverage = graded concepts / total concepts.
- Performance (`get_performance_score`): per concept, `appCorrect/appTotal` accuracy;
  estimate = weight-weighted mean over concepts with >=1 attempt; 95% interval from
  the pooled proportion (`scores::proportion_interval`). Coverage = concepts with an
  attempt / total concepts.
- Readiness (`get_readiness_score`): projects Performance onto 472..528 as
  `472 + performance*56`, rounded; the interval is the projected Performance interval
  widened by `(1 - coverage) * 8` points per side, clamped to `[472, 528]`. Abstains
  whenever Performance abstains.
- Abstain gate (each score shows no number until eligible): Memory needs
  `MIN_GRADED_REVIEWS = 5` and `MIN_COVERAGE_PCT = 0.25` (`memory_score.rs`);
  Performance needs `PERF_MIN_GRADED_ATTEMPTS = 5` and `PERF_MIN_COVERAGE_PCT = 0.25`
  (`performance_score.rs`); Readiness is eligible iff Performance is. When ineligible,
  `ScoreEnvelope::abstained` / the Memory equivalent zero the estimate + range and set
  `abstain_reason` (`scores.rs`). Thresholds are named constants, tunable.
- Per-subject breakdown (`get_speedrun_score_breakdown`): one row per authored leaf
  (memory + application inputs), for the study screen's per-subject table.

Dashboard host: the `speedrun-study` overview (dials, concept tree, per-subject
`SubjectTable` in `ts/routes/speedrun-study/`). The `speedrun-dashboard` route + Qt
dialog were removed; `ts/routes/speedrun-dashboard/lib.ts` + `StageGlyph.svelte` are
kept as the shared score/stage library the study screen imports.

### Authoring UI

`ts/routes/speedrun-decks/` (home) and `ts/routes/speedrun-hierarchy/` (tree plus
concept/problem editors). Backend: the `authoring.rs` RPCs. Autosave was removed
(`createAutosave`/`AUTOSAVE_MS` gone): the editor now has an explicit Save button with
a `SaveStatus`/`saveStatusLabel`/`hasUnsavedChanges` readout (`lib.ts`, tested in
`lib.test.ts`) and an unsaved-changes guard (beforeunload + back-nav). The hierarchy
still lives in config (unlike mastery, now on cards), so it is guarded by the explicit
Save plus the account screen's "sync before switching devices" nudge.
Dropping the autosave churn also fixed the multi-branch selection highlight (removed
the `:focus-within` coral trigger; only the open leaf's direct parent branch
highlights) and the text-selection flashes.

### Demo mode

Tools menu -> "Speedrun: Demo study screens" (`qt/aqt/speedrun_demo.py`) ->
`ts/routes/speedrun-review-demo/` (mock data, no backend).

### Seed

`rslib/src/speedrun/seed.rs` (RPC `SpeedrunEnsureSeeded` -> `{seeded, updated}`)
preloads two bundled decks — a biochem demo and the MCAT chapters 31-35 deck — from
JSON compiled in (`seed_data/*.json`), reusing the same `speedrun_save_hierarchy` +
`speedrun_next_card` the UI drives (no per-platform seed code). Idempotent and
fail-open per deck, so it is safe on every open: a missing deck is created, and a
bundled deck whose stored tree has **fewer concepts than the current bundle** is
re-stored + reconciled in place (keyed by `conceptId`, so existing cards/progress
survive) — the backfill that fixed the MCAT Physics topics showing 0 concepts (deck is
now 5 topics / 20 concepts / 80 problems). A user-grown deck (>= bundle) is never
touched. Desktop seeds on collection open (`qt/aqt/main.py` `_maybe_seed_speedrun`);
the mobile shell seeds from the decks-screen load (`ensureSeeded` in
`speedrun-hierarchy/lib.ts`). The old Python seed modules (`seed_deck.py`,
`seed_mcat_ch31_35*.py`) and their tests were removed.

### Accounts + sync

Accounts use Anki's **own** native sync login against the in-repo self-hosted sync
server (`SimpleServer`); there is no third-party identity provider. The account screen
(`ts/routes/speedrun-account/`) collects a username, password, and server endpoint
(default `http://127.0.0.1:8080/`, `SimpleServer`'s port) and the host runs
`col.sync_login(username, password, endpoint)`, stores the returned `hkey` in the
desktop profile (`pm.set_sync_key` / `set_sync_username` / `set_custom_sync_url`), and
lets Anki's normal sync (button + auto-sync on open/close) run against it. Sign out
clears the stored key. The server authenticates the static `SYNC_USER*` accounts it is
configured with (`get_host_key` in `rslib/src/sync/http_server/mod.rs`, one stable
per-user folder each); there is no lazy provisioning. Reached in-app from the profile
avatar on the Decks home (`goto("/speedrun-account")` inside the main `speedrunWeb`
webview, no separate window) + `mediasrv.py` handlers (`speedrunSyncLogin` / `SyncNow`
/ `SignOut` / `SyncStatus`, using the protobuf `_speedrun_request`/`_speedrun_response`
convention) declared on `FrontendService` (`proto/anki/frontend.proto`).

### AI deck-import

`rslib/src/speedrun/ai_import.rs` (RPCs `SpeedrunAiConfig` / `SpeedrunAiImport`) turns
uploaded source text into a validated authored hierarchy. The OpenAI Chat Completions
call is **server-side in Rust** (reqwest), so the key never reaches the client. The key
is developer-supplied, resolved by `ai_import.rs::pick_key`: a non-empty `OPENAI_API_KEY`
env var wins, else the in-code constant `OPENAI_API_KEY` (kept empty), else it stays
unconfigured. Preferred setup is a gitignored `.env` at the repo root (`OPENAI_API_KEY=sk-...`,
copied from the committed `.env.example`), loaded once at backend startup by
`dotenvy::dotenv()` in `backend/mod.rs::init_backend` so it populates that env var (a real
exported env var still wins; a missing `.env` is a no-op). `.env` is gitignored — never commit
a real key. The model is the hardcoded `DEFAULT_AI_MODEL` (no user config). `SpeedrunAiConfig` just reports `{ available }`; with no key,
`SpeedrunAiImport` returns a plain `{kind:"error"}` ("AI import isn't configured") and the
rest of the app keeps working. Source text is treated as untrusted (randomly-delimited
block + "data, not instructions" system prompt), and the model's deck JSON is rebuilt
against the authoring schema (ids minted, every problem forced to four valid choices,
empty output rejected). The frontend (`ts/routes/speedrun-import/`: Intake/Chat/DeckPreview)
extracts file text in the browser, runs the clarify chat, then hands the result to the
builder via `SpeedrunSaveHierarchy` for review. Reached in-app from the New deck action on
the Decks home (`goto("/speedrun-import")`, no separate window); Intake also links to the
blank builder for manual authoring. To enable AI import, put your key in a gitignored `.env`
at the repo root (copy `.env.example`).

### Qt integration points

`qt/aqt/speedrun_demo.py` (dev-only demo dialog hosting a SvelteKit page) is the only
remaining Tools-menu entry; account + sync and AI deck-import moved in-app (the
profile avatar and New deck action on the Decks home, both `goto` inside
`speedrunWeb`), so `qt/aqt/speedrun_account.py` and `qt/aqt/speedrun_import.py` were
removed. Other integration points: patches in `main.py` (the `speedrunStudy` state,
the `speedrunWeb` webview, `_active_webview` for the background-op fade,
`updateTitleBar` = "Speedrun", `_maybe_seed_speedrun` on collection open),
`overview.py`, `deckbrowser.py` (both slimmed to load their SvelteKit page; the legacy
stdHtml renderers are gone), `mediasrv.py` (also the native account/sync handlers),
`webview.py`, and `toolbar.py`. `reviewer.py` no longer has any Speedrun code (the
classic reviewer serves only ordinary decks). `qt/aqt/speedrun.py` and
`speedrun_dashboard.py` were removed.

### RPCs

Data RPCs in `proto/anki/scheduler.proto`; Qt navigation in `proto/anki/frontend.proto`
(`SpeedrunOpenDeck/StartStudy/OverviewAction/ShowDecks`). The typed score RPCs
(`GetMemoryScore`, `GetPerformanceScore`, `GetReadinessScore`,
`GetSpeedrunScoreBreakdown`) and the JSON study/authoring/seed/AI RPCs
(`SpeedrunStudyState/NextCard/AnswerCard/RecordLearned/StudyHierarchy`,
`SpeedrunListDecks/GetHierarchy/SaveHierarchy/DeleteDeck/StudySummary`,
`SpeedrunEnsureSeeded`, `SpeedrunAiConfig/AiImport`) are implemented in
`rslib/src/scheduler/service/speedrun.rs` (thin delegators in `service/mod.rs`) and
delegate to `rslib/src/speedrun/`. The JSON RPCs stay `generic.Json` (deeply nested +
dynamic shapes matching the screens' `lib.ts`). Accounts/sync is **not** a shared-engine
RPC: `frontend.proto` declares `SpeedrunSyncLogin/SyncNow/SignOut/SyncStatus` as
Qt-handled `generic.Json` calls that `mediasrv.py` implements to drive Anki's native
sync login (no Rust impl, like the navigation RPCs). The
retired note-engine RPCs (`GetTopicGroupedQueue`, `GetSpeedrunCardMode/Context`,
`SpeedrunRecordAnswer`, `GetSpeedrunProgress`) were removed.

## Speedrun file index (by layer)

- Rust: `rslib/src/speedrun/` (taxonomy, card_signals, study, authoring, scores,
  memory_score, performance_score, readiness_score, score_breakdown, `ai_import`,
  `seed` + `seed_data/*.json`) plus the RPC bodies in
  `rslib/src/scheduler/service/speedrun.rs`. `parse_id` is shared in `speedrun/mod.rs`.
  Native accounts authenticate against `SimpleServer` (`sync/http_server/mod.rs`, static
  `SYNC_USER*` password login). Removed: `progression.rs`,
  `scheduler/queue/topic_grouped.rs`.
- Python: no Speedrun engine module left (`pylib/anki/speedrun/` is docstring-only);
  accounts reuse `collection.py`'s `sync_login`; score wrappers in
  `pylib/anki/scheduler/v3.py`.
- Qt: `qt/aqt/speedrun_{demo,account,import}.py` plus the patches listed above.
- TS: `ts/routes/speedrun-{decks,hierarchy,study,review,review-demo,account,import}/`;
  `ts/routes/speedrun-dashboard/` is now lib-only (`lib.ts` + `StageGlyph.svelte`,
  no route). Shared JSON helpers (`enc`/`dec`/`quiet`) live in
  `speedrun-hierarchy/lib.ts`. Tokens in `ts/lib/sass/speedrun-tokens.scss`
  (`speedrun-review/sr-tokens.scss` forwards to it); the accent token is `--sr-signal`.
- Android: host bridges in `android/app/.../pages/` (`PostRequestHandler.kt` routes the
  sync + AI-import + seed calls, `SpeedrunFragment.kt`/`PageFragment.kt` add the
  file-picker via `onShowFileChooser`, `PageWebViewClient.kt` whitelists the routes) and
  raw engine wrappers in `libanki/.../sched/Scheduler.kt`; `android/run` +
  `bridge/build_rust`.
- Proto: `proto/anki/scheduler.proto`, `proto/anki/frontend.proto`.

## Tests

- Rust: inline `#[cfg(test)]` in `rslib/src/speedrun/*` (shared fixtures in
  `study.rs` `mod testing`; `ai_import.rs` uses a fake `ChatClient`, `seed.rs` covers
  the backfill); run `cargo test -p anki speedrun`.
- Python: no Speedrun-specific tests remain (the seed-module tests were removed with
  their modules).
- TS: `ts/routes/speedrun-*/lib.test.ts` via `just test-ts` (now including
  `speedrun-import` and `speedrun-hierarchy`). E2E authored study flow in
  `ts/tests/e2e/speedrun.test.ts` via `just test-e2e` (drives the mobile-shell
  client-side routing against the auto-seeded demo deck).

## Deliberately out of scope

Two hygiene items were considered and intentionally not done (a scope decision, not
a pending TODO):

- Speedrun UI chrome strings stay hardcoded in the Svelte components (and the few
  Python entry points); they are not routed through `ftl/`.
- The dev-only demo route (`ts/routes/speedrun-review-demo/`, opened from the Tools
  menu via `qt/aqt/speedrun_demo.py` and from the mobile decks-home nav) still ships in
  the SvelteKit bundle; it is not gated behind a dev flag.

## Conventions and gotchas

- rslib errors use `error/mod.rs` `AnkiError`/`Result` plus snafu (no anyhow in
  rslib). Other Rust modules use anyhow with context.
- User-facing strings go through ftl. Prefer typed proto over `generic.Json`.
- `out/` is generated; do not edit it.
- Speedrun card visuals are Svelte components under `ts/routes/speedrun-*`; the
  `SpeedrunItem` note type is a minimal display shell (not a card-template engine) that
  also carries the concept's mastery in its `custom_data`.
