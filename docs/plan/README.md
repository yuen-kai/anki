# Speedrun: project memory (front door)

> The orientation doc for this fork. Read this first, then follow the authority order below. Maintained continuously (unlike the frozen PRD/specs).

## What this is

A fork of Anki that adds a desktop + mobile study app for **one exam (MCAT)**, built around three honestly-separated scores (memory / performance / readiness) and an explicit application-teaching flow: one **Study** button driven by a four-state per-topic mastery progression. Full intent in [`prd-speedrun.md`](prd-speedrun.md).

## Current state (2026-06-30)

- **Planning:** complete. PRD + 6 specs + decision log (D1–D32) written and frozen.
- **Build bring-up:** base Anki runs on desktop (`just run`, after installing `n2`); base AnkiDroid runs on an arm64 emulator. Zero source edits. Anki HEAD `b00308e`.
- **Implementation:** Phase 1 merged to `main` (AAMC taxonomy Rust module + Learn content layer / note types + seed). Integrated tests green: 4 Rust (`cargo test -p anki speedrun`) + 22 Python (`python3 -m unittest test_speedrun_content`). Code-reviewed (no Critical; 2 Important fixes landed: concept-seed idempotency, `</script>` JSON hardening). Phase 2a merged: topic-grouped Learn queue + `GetTopicGroupedQueue` RPC (10 Rust tests green on `main`; full pylib + aqt suite green; reviewed (no Critical, ready for 2b; perf B013 + undo-proof B015 tracked). Phase 2b merged: honest Memory score (`GetMemoryScore`) + give-up rule + the topic-queue undo proof (Rust memory_score 6 + topic_grouped 7 green on `main`; pylib/aqt green; reviewed; closing gate: all code checks pass (clippy, cross-language tests, mypy, rustfmt), only pre-existing markdown dprint residue remains, B018). **Phase 2 closed.** Phase 3 merged: Memory dashboard (Tools menu), Learn/Practice deck entry, and the fail-open application-card answer gate (28 gating tests; integrated verify green; reviewed, ready to close, no Critical; follow-ups B022-B024). **Desktop slice (Phases 1-3) complete.** **Phase 4 done:** the modified engine runs inside AnkiDroid on the emulator with the new RPCs verified on-device ([B001](backlog.md) resolved, [D29](decisions.md#d29)). **All four phases complete.** Remaining (tracked, non-blocking): B025 (AnkiDroid UI calls the new RPCs + on-device Speedrun review), desktop clean-machine installer, plus the Friday/Sunday milestones (AI, sync, evals). (B019 Learn-surfaces-new-cards: fixed.)
- **Post-phase (mastery progression):** the four-state per-topic progression is merged. The two Learn/Practice buttons are now **one Study button**, each card shows its **topic breadcrumb** (U1/U2), the correctness findings **F1-F3** ([B026-B028](backlog.md)) are fixed, and the dashboard has a **Topics view** (U3): a four-stage mastery ladder per topic, grouped by the AAMC hierarchy. The **U4** polish then unified the whole surface on one product accent (the cards' teal, theme-aware) across the dashboard + topics ladder, fixed the Study button's lost primary style, and retitled the dashboard window. All merged + verified on `main`: Rust 33 speedrun + 13 topic-queue, 184 template checks, pylib 36 + qt 42, ts 3, all green. Every tracked requirement (P1-P9, U1-U4, F1-F3) is now done; the one open follow-up is a live visual QA pass ([B029](backlog.md#b029): no browser in this sandbox). Tracked in [requirements.md](requirements.md).

## Doc map

| Doc | Role | Lifecycle |
| :-- | :-- | :-- |
| `README.md` (this) | Front door + current state + authority order | living |
| [`prd-speedrun.md`](prd-speedrun.md) | User-facing contract | frozen |
| [`spec-*.md`](.) | Implementation design (taxonomy, engine queue, study model, scores, mobile) | frozen |
| [`decisions.md`](decisions.md) | Decision log D1–D32 (Chose / Considered / Gaps) | append-only |
| [`design-iterations.md`](design-iterations.md) | Dated, feedback-driven design changes | append-only |
| [`backlog.md`](backlog.md) | Bugs / refactors / open issues | updated in place |
| [`requirements.md`](requirements.md) | Requirements ledger (every ask → status), reconcile before "done" | living |

## Authority order (what wins when docs conflict)

1. **This README** (current state)
2. **[decisions.md](decisions.md)**: latest non-superseded entry
3. **[design-iterations.md](design-iterations.md)**
4. **PRD / specs**: original intent, superseded wherever a later decision says so

The PRD/specs are frozen initial plans. Where a decision conflicts with them, the decision wins.

## Overrides since the plan

- spec-scores §7/§9.5 said `AttemptLog` persists from Wednesday → deferred to Phase 3/Friday ([D25](decisions.md#d25)); the Memory score uses revlog in the interim.
- spec-study-model §4-§6 (flat Learn/Practice two-mode model) → superseded by the four-state mastery progression ([D30](decisions.md#d30)/[D31](decisions.md#d31)/[D32](decisions.md#d32), [`spec-mastery-progression`](spec-mastery-progression.md)); merged + verified on `main`. The two-button Learn/Practice entry ([D20](decisions.md#d20)) is superseded by a single Study button ([D30](decisions.md#d30)); each card now shows its topic breadcrumb (U1/U2), merged + verified.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
