// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Performance score** (spec-scores §7, decision D33).
//!
//! Performance answers "can I get a *new, exam-style* question right?" from the
//! student's real accuracy on `SpeedrunApplication` cards (the application
//! problems), weighted by exam weight, with the shared evidence envelope and the
//! give-up rule. It is deliberately a *different* construct from Memory (recall
//! of a taught fact), so the two are never conflated (D7).
//!
//! NOTE: stub implementation — abstains until the Phase-6 engine work lands.

use crate::prelude::*;
use crate::speedrun::scores::ScoreEnvelope;
use crate::speedrun::scores::ScoreFormat;

impl Collection {
    /// Compute the Performance score for `deck_id` (and its children).
    pub(crate) fn get_performance_score(&mut self, _deck_id: DeckId) -> Result<ScoreEnvelope> {
        Ok(ScoreEnvelope::abstained(
            0.0,
            0,
            "performance score not yet computed".to_string(),
            ScoreFormat::Ratio,
        ))
    }
}
