# Speedrun: requirements ledger

> Every stated requirement, tracked to verified done. Reconcile this against the build before claiming any batch complete (see the `requirements-tracking` rule). Status: `todo` / `building` / `done` / `superseded`. "done" needs a test, a verified run, or a concrete file ref, not an assertion.

## Mastery progression (from the restatement, user-confirmed)

| # | Requirement (observable) | Status | Evidence |
| :-- | :-- | :-- | :-- |
| P1 | Cards sorted by the same AAMC hierarchy the scaffold uses | done | `topic_grouped.rs` groups by taxonomy topic; scaffold options come from the same tree |
| P2 | Track which hierarchy areas the user has learned (per-topic state) | done | `progression.rs` config map; `get_speedrun_progress` RPC. (Viewing it = U3.) |
| P3 | Blocked section: learn one hierarchy block at a time | done | `select_blocked_or_mixed`; test `blocked_phase_serves_one_learning_block_then_mixes` |
| P4 | Once a block is learned, mix it into the interleaved pool | done | same; blocked→mixed transition |
| P5 | Per-topic 4 states: learning, practicing, hierarchy, mastering | done | `progression.rs` state machine + tests |
| P6 | Upgrade the cards to match the state | done | `resolve_card_mode` + 4 card modes |
| P7 | Practicing: practice the concept | done | `concept_practice` mode (templates + tests) |
| P8 | Hierarchy: practice problems using the scaffold | done | `application_scaffolded` mode |
| P9 | Mastering: practice problems without the scaffold | done | `application_unscaffolded` mode (scaffold fades) |

## Recent UX asks

| # | Requirement (observable) | Status | Evidence / owner |
| :-- | :-- | :-- | :-- |
| U1 | Show the topic's hierarchy (breadcrumb) on the card, at least when learning | done | `GetSpeedrunCardContext` RPC + `taxonomy::topic_path_labels`; reviewer injects `window.speedrunTopicPath`; templates render it; tests: rust `topic_path_labels_walks_root_to_leaf`, node breadcrumb checks (184), qt 53 |
| U2 | Merge the Learn and Practice buttons into one | done | `overview.py` single Study button (`tr.studying_study()` -> `_start_session()`, state-aware queue w/ fallback); D20 superseded by D30; qt `test_speedrun.py` (53) |
| U3 | A UI listing all topics and which of the 4 stages each is at | done | Topics section on the dashboard: `get_speedrun_progress` (now carries the hierarchy `path`, web-exposed) → `buildTopicsView` groups by category + a per-topic 4-stage mastery track + ladder summary. Tests: `lib.test.ts` (3), rust `progress_lists_covered_topics_with_states` |
| U4 | `frontend-design` polish pass on the Speedrun UI (dashboard, cards, Study entry, topics view) | todo | after U1/U2/U3 + F1-F3 land; follow the `frontend-design` skill (distinct identity, type, signature element) |

## Correctness fixes from the progression review (binding)

| # | Requirement | Status | Ref |
| :-- | :-- | :-- | :-- |
| F1 | Reset `window.speedrunCardMode` between cards (no leak) | done | [B027](backlog.md#b027) fixed: `card_context_inject_script` always resets both globals; qt test `test_nothing_to_inject_resets_both_globals` |
| F2 | `speedrun_record_answer` ignores non-Speedrun note types | done | [B028](backlog.md#b028) fixed: early-return on `note_kind == Other`; rust test `record_answer_ignores_non_speedrun_note_kind` |
| F3 | The single Study flow must not starve graduated topics' due reviews | done | [B026](backlog.md#b026) fixed: blocked phase also serves graduated due reviews; rust test `blocked_phase_serves_graduated_reviews_but_withholds_other_new_topics` |

## Audit (2026-06-30)

Reconciled the progression restatement + every later message. Initially-missed **U1, U2** are built, merged, and verified green (the refinement subagent's network died mid-run; its finished-but-uncommitted work was salvaged from the worktree, verified, and merged). Review-found correctness **F1, F2, F3** are now fixed + tested. Remaining: **U3** (topics-state UI), then **U4** polish. Everything in the restatement (P1-P9) is built and verified. No other dropped requirement found. Foundation milestones (Phases 1-4) tracked in [README](README.md) + the phase todos.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
