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
//!   over in-scope cards, with an evidence envelope and the give-up rule
//!   (Phase 2b).

pub mod card_signals;
pub mod memory_score;
pub mod taxonomy;
