# Speedrun — project memory (front door)

> The orientation doc for this fork. Read this first, then follow the authority order below. Maintained continuously (unlike the frozen PRD/specs).

## What this is

A fork of Anki that adds a desktop + mobile study app for **one exam (MCAT)**, built around three honestly-separated scores (memory / performance / readiness) and an explicit application-teaching flow (Learn vs Practice). Full intent in [`prd-speedrun.md`](prd-speedrun.md).

## Current state (2026-06-30)

- **Planning:** complete. PRD + 5 specs + decision log (D1–D21) written and frozen.
- **Build bring-up:** base Anki runs on desktop (`just run`, after installing `n2`); base AnkiDroid runs on an arm64 emulator. Zero source edits. Anki HEAD `b00308e`.
- **Implementation:** Phase 1 merged to `main` (AAMC taxonomy Rust module + Learn content layer / note types + seed). Integrated tests green: 4 Rust (`cargo test -p anki speedrun`) + 22 Python (`python3 -m unittest test_speedrun_content`). Code-reviewed (no Critical; 2 Important fixes landed: concept-seed idempotency, `</script>` JSON hardening). Phase 2a merged: topic-grouped Learn queue + `GetTopicGroupedQueue` RPC (10 Rust tests green on `main`; full pylib + aqt suite green; reviewed (no Critical, ready for 2b; perf B013 + undo-proof B015 tracked). Phase 2b merged: honest Memory score (`GetMemoryScore`) + give-up rule + the topic-queue undo proof (Rust memory_score 6 + topic_grouped 7 green on `main`; pylib/aqt green; reviewed, ready to close; `just check` final pass running). Next: Phase 3 (desktop UX: Learn/Practice, reviewer hook, Memory dashboard).

## Doc map

| Doc | Role | Lifecycle |
| :-- | :-- | :-- |
| `README.md` (this) | Front door + current state + authority order | living |
| [`prd-speedrun.md`](prd-speedrun.md) | User-facing contract | frozen |
| [`spec-*.md`](.) | Implementation design (taxonomy, engine queue, study model, scores, mobile) | frozen |
| [`decisions.md`](decisions.md) | Decision log D1–D21 (Chose / Considered / Gaps) | append-only |
| [`design-iterations.md`](design-iterations.md) | Dated, feedback-driven design changes | append-only |
| [`backlog.md`](backlog.md) | Bugs / refactors / open issues | updated in place |

## Authority order (what wins when docs conflict)

1. **This README** (current state)
2. **[decisions.md](decisions.md)** — latest non-superseded entry
3. **[design-iterations.md](design-iterations.md)**
4. **PRD / specs** — original intent, superseded wherever a later decision says so

The PRD/specs are frozen initial plans. Where a decision conflicts with them, the decision wins.

## Overrides since the plan

- spec-scores §7/§9.5 said `AttemptLog` persists from Wednesday → deferred to Phase 3/Friday ([D25](decisions.md#d25)); the Memory score uses revlog in the interim.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
