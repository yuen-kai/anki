// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! FSRS retrievability of a card, shared by the Memory score and the
//! per-subject breakdown so the two never compute recall differently.
//!
//! Read-only: computes over already-stored card state and never mutates the
//! collection.

use fsrs::FSRS;
use fsrs::FSRS5_DEFAULT_DECAY;

use crate::card::Card;
use crate::scheduler::timing::SchedTimingToday;

/// Retrievability assigned to a card that carries no FSRS memory state (e.g. an
/// unreviewed card, an SM-2 card, or one moved with "set due date"). A mild
/// "probably still known" prior so such cards never read as forgotten.
pub(crate) const NO_MEMORY_STATE_RETRIEVABILITY: f32 = 0.9;

/// FSRS current retrievability for a card in `[0, 1]`, or the
/// [`NO_MEMORY_STATE_RETRIEVABILITY`] prior when the card has no memory state.
/// This is the same computation the stats graphs use; a card's weakness is
/// `1 - this`.
pub(crate) fn card_retrievability(card: &Card, timing: &SchedTimingToday, fsrs: &FSRS) -> f32 {
    match card.memory_state {
        Some(state) => {
            let elapsed_seconds = card.seconds_since_last_review(timing).unwrap_or_default();
            fsrs.current_retrievability_seconds(
                state.into(),
                elapsed_seconds,
                card.decay.unwrap_or(FSRS5_DEFAULT_DECAY),
            )
        }
        None => NO_MEMORY_STATE_RETRIEVABILITY,
    }
}
