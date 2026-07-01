// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Shared evidence envelope for the Speedrun **Performance** and **Readiness**
//! scores (spec-scores §4, decision D33). Memory keeps its own
//! [`crate::speedrun::memory_score`] module; these two share this envelope so
//! all three scores render through the same contract, never blended into one
//! number (D7).
//!
//! The real computations live in [`crate::speedrun::performance_score`] and
//! [`crate::speedrun::readiness_score`]; this module only defines the shared
//! shape and the abstaining constructor, so a score with no evidence produces a
//! silent (numberless) envelope instead of a guess in a nice font.

use crate::speedrun::memory_score::Confidence;
use crate::timestamp::TimestampSecs;

/// How the UI should render a score's numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreFormat {
    /// A probability in `[0, 1]` (Memory, Performance).
    Ratio,
    /// An integer on the 472-528 MCAT scale (Readiness).
    Points,
}

impl ScoreFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            ScoreFormat::Ratio => "ratio",
            ScoreFormat::Points => "points",
        }
    }
}

/// The shared evidence envelope (spec §4) for Performance and Readiness. Same
/// fields as [`crate::speedrun::memory_score::MemoryScore`] plus a `format`, so
/// the three scores render identically. When `abstained`, `estimate`/`range_*`
/// are 0 and `abstain_reason` names what is missing.
#[derive(Debug, Clone)]
pub struct ScoreEnvelope {
    pub estimate: f32,
    pub range_low: f32,
    pub range_high: f32,
    pub coverage_pct: f32,
    pub confidence: Confidence,
    pub updated_at_secs: i64,
    pub reasons: Vec<String>,
    pub abstained: bool,
    pub abstain_reason: String,
    pub graded_reviews: u32,
    pub format: ScoreFormat,
}

impl ScoreEnvelope {
    /// A silent (numberless) envelope: the give-up rule fired. Coverage and the
    /// graded count are still reported honestly; the estimate/range are zeroed.
    pub fn abstained(
        coverage_pct: f32,
        graded_reviews: u32,
        abstain_reason: String,
        format: ScoreFormat,
    ) -> Self {
        ScoreEnvelope {
            estimate: 0.0,
            range_low: 0.0,
            range_high: 0.0,
            coverage_pct,
            confidence: Confidence::Low,
            updated_at_secs: TimestampSecs::now().0,
            reasons: Vec::new(),
            abstained: true,
            abstain_reason,
            graded_reviews,
            format,
        }
    }
}
