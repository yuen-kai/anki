// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Speedrun: MCAT study extensions on top of Anki.
//!
//! All studying flows through the single **authored-hierarchy engine**: an
//! authored topic/concept/problem tree ([`authoring`]) whose concepts are
//! materialized into FSRS-scheduled cards and driven by the `speedrun-review`
//! screen ([`study`]). The scores read that same engine, so studying moves
//! them.
//!
//! - [`taxonomy`]: the AAMC MCAT topic taxonomy (seed subset) — pure data used
//!   only for score weighting and labels, not a study path.
//! - [`card_signals`]: FSRS retrievability of a materialized concept card,
//!   shared by the Memory score and the per-subject breakdown.
//! - [`memory_score`]: the honest Memory score — aggregated FSRS retrievability
//!   over the deck's concept cards, rolled up the authored hierarchy, with an
//!   evidence envelope and the give-up rule.
//! - [`scores`]: the shared evidence envelope for the Performance and Readiness
//!   scores, so all three render identically and are never blended.
//! - [`performance_score`]: accuracy over the authored concepts' application
//!   attempts.
//! - [`readiness_score`]: Performance projected onto the 472-528 scale.
//! - [`score_breakdown`]: per-authored-leaf Memory/Performance inputs for the
//!   study screen's per-subject details modal.
//! - [`study`]: the `speedrun-review` study screen's backend — authored
//!   concepts materialized into FSRS-scheduled cards, plus the per-concept
//!   mastery store, shared by the desktop and AnkiDroid webviews.
//! - [`authoring`]: the authoring store CRUD (list decks, get/save/delete the
//!   hierarchy blob) shared by the desktop and AnkiDroid deck/authoring
//!   screens.

pub mod authoring;
pub mod card_signals;
pub mod memory_score;
pub mod performance_score;
pub mod readiness_score;
pub mod score_breakdown;
pub mod scores;
pub mod study;
pub mod taxonomy;

use crate::prelude::*;

/// Parse a numeric id the Speedrun screens send as a string, naming the field
/// in the error. Shared by the authoring store and the service RPC layer so id
/// parsing lives in one place.
pub(crate) fn parse_id(value: &str, what: &str) -> Result<i64> {
    match value.parse::<i64>() {
        Ok(id) => Ok(id),
        Err(_) => invalid_input!("invalid {what}: {value}"),
    }
}
