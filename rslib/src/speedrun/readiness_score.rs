// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Readiness score**.
//!
//! Readiness answers "what would I score on the 472-528 scale today?" by
//! projecting the [Performance score](crate::speedrun::performance_score) onto
//! the MCAT total-score scale, widening the range as topic coverage shrinks. It
//! abstains whenever Performance abstains (there is nothing to project), and it
//! is explicitly a projection *from practice*, not a number calibrated against
//! real exam outcomes, which its `reasons` say.
//!
//! - `estimate` = `472 + performance · 56`, rounded to a whole score.
//! - `range` = the Performance interval projected the same way, then widened by
//!   `(1 - coverage) · COVERAGE_WIDENING_POINTS` on each side and clamped to
//!   the scale, so a thinly-covered deck reads as a wider, less certain band.
//! - `coverage_pct`, `graded_reviews`, `confidence` pass through from
//!   Performance; `format` is `points` so the UI renders whole scores.
//!
//! Read-only: it only calls the (read-only) Performance score.

use crate::prelude::*;
use crate::speedrun::scores::pct_round;
use crate::speedrun::scores::ScoreEnvelope;
use crate::speedrun::scores::ScoreFormat;
use crate::timestamp::TimestampSecs;

/// The MCAT total-score scale (inclusive), used to project Performance.
pub const MCAT_SCORE_MIN: f32 = 472.0;
pub const MCAT_SCORE_MAX: f32 = 528.0;
/// The width of the scale (528 - 472).
const MCAT_SCORE_SPAN: f32 = MCAT_SCORE_MAX - MCAT_SCORE_MIN;
/// Points added to each side of the range at zero coverage, scaling to nothing
/// at full coverage. Tunable: the honest "we've seen little of the exam"
/// widening the projected band.
const COVERAGE_WIDENING_POINTS: f32 = 8.0;

/// Project a `[0, 1]` performance value onto the MCAT scale.
fn to_scale(ratio: f32) -> f32 {
    MCAT_SCORE_MIN + ratio * MCAT_SCORE_SPAN
}

impl Collection {
    /// Compute the Readiness score for `deck_id` (and its children).
    pub(crate) fn get_readiness_score(&mut self, deck_id: DeckId) -> Result<ScoreEnvelope> {
        let performance = self.get_performance_score(deck_id)?;

        // Nothing to project while Performance is silent; carry its scope + say
        // Performance is the blocker.
        if performance.abstained {
            return Ok(ScoreEnvelope::abstained(
                performance.coverage_pct,
                performance.graded_reviews,
                format!(
                    "needs the Performance score first: {}",
                    performance.abstain_reason
                ),
                ScoreFormat::Points,
            ));
        }

        let widen = (1.0 - performance.coverage_pct) * COVERAGE_WIDENING_POINTS;
        let mut estimate = to_scale(performance.estimate).round();
        let range_low = (to_scale(performance.range_low) - widen)
            .round()
            .clamp(MCAT_SCORE_MIN, MCAT_SCORE_MAX);
        let range_high = (to_scale(performance.range_high) + widen)
            .round()
            .clamp(MCAT_SCORE_MIN, MCAT_SCORE_MAX);
        // Rounding can nudge the point estimate a hair outside its widened band.
        estimate = estimate.clamp(range_low, range_high);

        // Coverage leads the readiness display ("Projected X, covered Y%…").
        let reasons = vec![
            format!("covered {}% of topics", pct_round(performance.coverage_pct)),
            "projected from your application accuracy".to_string(),
            "not calibrated to real MCAT outcomes yet".to_string(),
        ];

        Ok(ScoreEnvelope {
            estimate,
            range_low,
            range_high,
            coverage_pct: performance.coverage_pct,
            confidence: performance.confidence,
            updated_at_secs: TimestampSecs::now().0,
            reasons,
            abstained: false,
            abstain_reason: String::new(),
            graded_reviews: performance.graded_reviews,
            format: ScoreFormat::Points,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speedrun::study::testing::set_hierarchy;

    /// A deck of `leaves` (each `(title, conceptId)`) whose concepts are each
    /// given `attempts_each` application attempts, `correct_each` of them
    /// right.
    fn seed_applied(
        col: &mut Collection,
        leaves: &[(&str, &str)],
        attempts_each: u32,
        correct_each: u32,
    ) {
        let hierarchy: Vec<(&str, &[&str])> = leaves
            .iter()
            .map(|(title, cid)| (*title, std::slice::from_ref(cid)))
            .collect();
        set_hierarchy(col, "1", &hierarchy);
        for (_title, cid) in leaves {
            col.speedrun_seed_app_counts(DeckId(1), cid, correct_each, attempts_each);
        }
    }

    const ALL_LEAVES: [(&str, &str); 8] = [
        ("Structure", "c-structure"),
        ("pKa & Titration", "c-pka"),
        ("Metabolism", "c-metabolism"),
        ("Levels of Structure", "c-levels"),
        ("Folding", "c-folding"),
        ("Kinetics", "c-kinetics"),
        ("Inhibition", "c-inhibition"),
        ("Regulation", "c-regulation"),
    ];

    /// An eligible deck yields a whole-number score inside a valid 472-528
    /// band, formatted as points.
    #[test]
    fn eligible_deck_projects_onto_the_scale() {
        let mut col = Collection::new();
        seed_applied(&mut col, &ALL_LEAVES[..4], 4, 3);

        let score = col.get_readiness_score(DeckId(1)).unwrap();

        assert!(!score.abstained);
        assert_eq!(score.format.as_str(), "points");
        assert!(
            MCAT_SCORE_MIN <= score.range_low
                && score.range_low <= score.estimate
                && score.estimate <= score.range_high
                && score.range_high <= MCAT_SCORE_MAX,
            "472 <= low <= est <= high <= 528, got [{}, {}, {}]",
            score.range_low,
            score.estimate,
            score.range_high
        );
        assert_eq!(score.estimate, score.estimate.round(), "a whole score");
        assert!(
            score.reasons.first().unwrap().contains("covered"),
            "coverage leads the readiness reasons, got {:?}",
            score.reasons
        );
    }

    /// When Performance abstains (thin data), Readiness abstains too and points
    /// at Performance as the blocker.
    #[test]
    fn abstains_when_performance_abstains() {
        let mut col = Collection::new();
        // Only 2 application attempts: Performance abstains → Readiness abstains.
        seed_applied(&mut col, &ALL_LEAVES[..1], 2, 2);

        let score = col.get_readiness_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert_eq!(score.estimate, 0.0);
        assert_eq!(score.format.as_str(), "points");
        assert!(
            score.abstain_reason.to_lowercase().contains("performance"),
            "reason names Performance as the blocker, got {:?}",
            score.abstain_reason
        );
    }

    /// Lower topic coverage widens the projected band (all else equal-ish).
    #[test]
    fn lower_coverage_widens_the_range() {
        // Full coverage, strong record.
        let mut wide_col = Collection::new();
        seed_applied(&mut wide_col, &ALL_LEAVES, 4, 3);
        let full = wide_col.get_readiness_score(DeckId(1)).unwrap();

        // Half coverage, same per-concept record. All eight leaves exist, but
        // only the first four are applied, so coverage is diluted without
        // changing accuracy.
        let mut narrow_col = Collection::new();
        let leaves: Vec<(&str, &[&str])> = ALL_LEAVES
            .iter()
            .map(|(t, c)| (*t, std::slice::from_ref(c)))
            .collect();
        set_hierarchy(&mut narrow_col, "1", &leaves);
        for (_t, cid) in &ALL_LEAVES[..4] {
            narrow_col.speedrun_seed_app_counts(DeckId(1), cid, 3, 4);
        }
        let half = narrow_col.get_readiness_score(DeckId(1)).unwrap();

        assert!(!full.abstained && !half.abstained);
        let full_width = full.range_high - full.range_low;
        let half_width = half.range_high - half.range_low;
        assert!(
            half_width > full_width,
            "half-covered band ({half_width}) should be wider than full-covered ({full_width})"
        );
    }

    /// Perfect application accuracy projects near the top of the scale;
    /// all-wrong projects near the bottom.
    #[test]
    fn extremes_map_to_scale_ends() {
        let mut top = Collection::new();
        seed_applied(&mut top, &ALL_LEAVES, 4, 4); // all correct, full coverage
        let top_score = top.get_readiness_score(DeckId(1)).unwrap();
        assert!(
            top_score.estimate >= 525.0,
            "perfect practice → near 528, got {}",
            top_score.estimate
        );

        let mut bottom = Collection::new();
        seed_applied(&mut bottom, &ALL_LEAVES, 4, 0); // all wrong, full coverage
        let bottom_score = bottom.get_readiness_score(DeckId(1)).unwrap();
        assert!(
            bottom_score.estimate <= 475.0,
            "no correct answers → near 472, got {}",
            bottom_score.estimate
        );
    }
}
