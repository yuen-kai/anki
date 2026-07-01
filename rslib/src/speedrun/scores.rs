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

use std::collections::HashMap;

use crate::speedrun::memory_score::Confidence;
use crate::speedrun::taxonomy::seed_taxonomy;
use crate::timestamp::TimestampSecs;

/// z for a 95% normal interval (shared with the Memory score's interval).
pub(crate) const Z_95: f32 = 1.96;

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
    /// The 95% interval of a proportion `p` over `n` trials, centered on
    /// `estimate` (`p ± Z_95·SE`), clamped to `[0, 1]`. `estimate` may differ
    /// from `p` (e.g. a weight-adjusted mean) while the width comes from the
    /// pooled proportion, so the reported number always sits inside its range.
    pub(crate) fn proportion_interval(estimate: f32, p: f32, n: u32) -> (f32, f32) {
        if n == 0 {
            return (estimate, estimate);
        }
        let half = Z_95 * (p * (1.0 - p) / n as f32).sqrt();
        ((estimate - half).clamp(0.0, 1.0), (estimate + half).clamp(0.0, 1.0))
    }

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

/// Two-band confidence from a graded count and coverage: `High` needs both at or
/// above the high thresholds, `Medium` both at or above the medium ones, else
/// `Low`. Shared so the Performance and Readiness scores band the same way; the
/// thresholds themselves are each score's own tunable constants.
pub(crate) fn confidence_from(
    graded: u32,
    coverage: f32,
    high_graded: u32,
    high_coverage: f32,
    medium_graded: u32,
    medium_coverage: f32,
) -> Confidence {
    if graded >= high_graded && coverage >= high_coverage {
        Confidence::High
    } else if graded >= medium_graded && coverage >= medium_coverage {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

/// In-scope leaf topic id → human label, for a score's `reasons`.
pub(crate) fn topic_labels() -> HashMap<String, String> {
    seed_taxonomy()
        .into_iter()
        .filter(|node| node.in_scope)
        .map(|node| (node.id, node.label))
        .collect()
}

/// Round a `[0, 1]` fraction to a whole-percent integer, for reason strings.
pub(crate) fn pct_round(fraction: f32) -> i64 {
    (fraction * 100.0).round() as i64
}
