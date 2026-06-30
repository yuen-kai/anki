// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Per-card Speedrun signals shared by the topic-grouped "Learn" queue (Phase
//! 2a) and the Memory score (Phase 2b): a card's FSRS retrievability and the
//! card→taxonomy-topic mapping. Both features must score and map a card the
//! same way, so the logic lives once here rather than being copied between the
//! queue and the score (decisions D8, D16, D23).
//!
//! Everything is read-only: these helpers compute over already-stored card and
//! taxonomy state and never mutate the collection.

use std::collections::HashMap;

use fsrs::FSRS;
use fsrs::FSRS5_DEFAULT_DECAY;

use crate::card::Card;
use crate::scheduler::timing::SchedTimingToday;
use crate::speedrun::taxonomy::seed_taxonomy;

/// Retrievability assigned to a card that carries no FSRS memory state (e.g. an
/// SM-2 card, or one moved with "set due date"). A mild "probably still known"
/// prior so such cards never masquerade as the weakest in the queue, nor read
/// as forgotten in the Memory score (decision D23).
pub(crate) const NO_MEMORY_STATE_RETRIEVABILITY: f32 = 0.9;

/// The in-scope leaf topics keyed by id with their exam weight, taken from the
/// seed taxonomy. Structural foundation/category nodes (`in_scope == false`)
/// are excluded, so the map's keys are exactly the scorable topic ids.
pub(crate) fn leaf_topic_weights() -> HashMap<String, f32> {
    seed_taxonomy()
        .into_iter()
        .filter(|node| node.in_scope)
        .map(|node| (node.id, node.exam_weight))
        .collect()
}

/// The taxonomy leaf a card maps to: the note tag exactly equal to an in-scope
/// leaf id, the smallest id winning on ties so the choice is deterministic;
/// `None` when no tag matches (an unmapped card). `leaf_weights` is the map
/// returned by [`leaf_topic_weights`].
pub(crate) fn card_topic(tags: &[String], leaf_weights: &HashMap<String, f32>) -> Option<String> {
    tags.iter()
        .filter(|tag| leaf_weights.contains_key(tag.as_str()))
        .min()
        .cloned()
}

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
