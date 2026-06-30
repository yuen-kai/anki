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

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
