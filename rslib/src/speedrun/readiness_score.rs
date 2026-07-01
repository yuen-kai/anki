// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Readiness score** (spec-scores §7, decision D33).
//!
//! Readiness answers "what would I score on the 472-528 scale today?" by
//! projecting the [Performance score](crate::speedrun::performance_score) onto
//! the MCAT scale, widening the range as topic coverage shrinks. It abstains
//! whenever Performance abstains, and it is explicitly a projection from
//! practice, not a number calibrated against real exam outcomes (that is the
//! Sunday milestone), which its reasons must say.
//!
//! NOTE: stub implementation — abstains until the Phase-6 engine work lands.

use crate::prelude::*;
use crate::speedrun::scores::ScoreEnvelope;
use crate::speedrun::scores::ScoreFormat;

/// The MCAT total-score scale (inclusive), used to project Performance.
pub const MCAT_SCORE_MIN: f32 = 472.0;
pub const MCAT_SCORE_MAX: f32 = 528.0;

impl Collection {
    /// Compute the Readiness score for `deck_id` (and its children).
    pub(crate) fn get_readiness_score(&mut self, _deck_id: DeckId) -> Result<ScoreEnvelope> {
        Ok(ScoreEnvelope::abstained(
            0.0,
            0,
            "readiness score not yet computed".to_string(),
            ScoreFormat::Points,
        ))
    }
}
