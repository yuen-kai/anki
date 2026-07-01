// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Speedrun: MCAT study extensions on top of Anki.
//!
//! - [`taxonomy`]: the AAMC MCAT topic taxonomy (seed subset) plus pure
//!   coverage/weakness helpers (Phase 1).
//! - [`card_signals`]: per-card FSRS retrievability and the card→topic mapping,
//!   shared by the topic-grouped queue and the Memory score so the two never
//!   diverge (Phase 2a/2b).
//! - [`memory_score`]: the honest Memory score — aggregated FSRS retrievability
//!   over in-scope cards, with an evidence envelope and the give-up rule (Phase
//!   2b).
//! - [`scores`]: the shared evidence envelope for the Performance and Readiness
//!   scores, so all three render identically and are never blended (D33).
//! - [`performance_score`]: accuracy over application (exam-style) cards (D33).
//! - [`readiness_score`]: Performance projected onto the 472-528 scale (D33).
//! - [`progression`]: the per-topic four-state mastery lifecycle
//!   (learning → practicing → hierarchy → mastering), its state-aware card-mode
//!   resolution and its config-backed transitions (decisions D30–D32).

pub mod card_signals;
pub mod memory_score;
pub mod performance_score;
pub mod progression;
pub mod readiness_score;
pub mod scores;
pub mod taxonomy;
