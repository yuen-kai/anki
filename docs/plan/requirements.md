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
| U1 | Show the topic's hierarchy (breadcrumb) on the card, at least when learning | building | UX refinement subagent (was missed initially; now tracked + in flight) |
| U2 | Merge the Learn and Practice buttons into one | building | UX refinement subagent (was missed initially) |
| U3 | A UI listing all topics and which of the 4 stages each is at | todo | build after U1/U2 (reuses taxonomy labels); data from `get_speedrun_progress` |

## Correctness fixes from the progression review (binding)

| # | Requirement | Status | Ref |
| :-- | :-- | :-- | :-- |
| F1 | Reset `window.speedrunCardMode` between cards (no leak) | todo | [B027](backlog.md#b027); post-UX fix pass |
| F2 | `speedrun_record_answer` ignores non-Speedrun note types | todo | [B028](backlog.md#b028); post-UX fix pass |
| F3 | The single Study flow must not starve graduated topics' due reviews | todo | [B026](backlog.md#b026); post-UX fix pass |

## Audit (2026-06-30)

Reconciled the progression restatement + every later message. Initially-missed: **U1, U2** (caught after the user flagged; now in flight). New ask: **U3**. Review-found correctness: **F1, F2, F3**. Everything in the restatement (P1-P9) is built and verified. No other dropped requirement found. Foundation milestones (Phases 1-4) tracked in [README](README.md) + the phase todos.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
