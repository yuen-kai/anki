// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Readiness score** (spec-scores §7, decision D33).
//!
//! Readiness answers "what would I score on the 472-528 scale today?" by
//! projecting the [Performance score](crate::speedrun::performance_score) onto
//! the MCAT total-score scale, widening the range as topic coverage shrinks. It
//! abstains whenever Performance abstains (there is nothing to project), and it
//! is explicitly a projection *from practice*, not a number calibrated against
//! real exam outcomes (that is the Sunday milestone), which its `reasons` say.
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
/// at full coverage. Tunable (D33): the honest "we've seen little of the exam"
/// widening the projected band.
const COVERAGE_WIDENING_POINTS: f32 = 8.0;

/// Project a `[0, 1]` performance value onto the MCAT scale.
fn to_scale(ratio: f32) -> f32 {
    MCAT_SCORE_MIN + ratio * MCAT_SCORE_SPAN
}

impl Collection {
    /// Compute the Readiness score for `deck_id` (and its children), spec §7.
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
            "not calibrated to real MCAT outcomes yet (the Sunday milestone)".to_string(),
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
    use crate::revlog::RevlogEntry;
    use crate::revlog::RevlogId;
    use crate::revlog::RevlogReviewKind;
    use crate::types::Usn;

    const STRUCTURE: &str = "mcat::biomolecules::amino_acids::structure";
    const PKA: &str = "mcat::biomolecules::amino_acids::pka_titration";
    const METABOLISM: &str = "mcat::biomolecules::amino_acids::metabolism";
    const STRUCTURE_LEVELS: &str = "mcat::biomolecules::proteins::structure_levels";
    const FOLDING: &str = "mcat::biomolecules::proteins::folding";
    const KINETICS: &str = "mcat::biomolecules::enzymes::kinetics";
    const INHIBITION: &str = "mcat::biomolecules::enzymes::inhibition";
    const REGULATION: &str = "mcat::biomolecules::enzymes::regulation";
    const ALL_TOPICS: [&str; 8] = [
        STRUCTURE,
        PKA,
        METABOLISM,
        STRUCTURE_LEVELS,
        FOLDING,
        KINETICS,
        INHIBITION,
        REGULATION,
    ];

    fn add_application_notetype(col: &mut Collection) -> Notetype {
        let mut nt = Notetype {
            name: "SpeedrunApplication".to_string(),
            ..Default::default()
        };
        nt.add_field("Front");
        nt.add_template("Card 1", "{{Front}}", "{{Front}}");
        col.add_notetype(&mut nt, true).unwrap();
        nt
    }

    fn seed_card(
        col: &mut Collection,
        nt: &Notetype,
        tag: &str,
        buttons: &[u8],
        next_id: &mut i64,
    ) -> CardId {
        let mut note = nt.new_note();
        note.set_field(0, "front").unwrap();
        note.tags.push(tag.to_string());
        col.add_note(&mut note, DeckId(1)).unwrap();
        let cid = col.storage.card_ids_of_notes(&[note.id]).unwrap()[0];
        for button in buttons {
            *next_id += 1;
            let entry = RevlogEntry {
                id: RevlogId(*next_id),
                cid,
                usn: Usn(-1),
                button_chosen: *button,
                interval: 1,
                last_interval: 1,
                ease_factor: 2500,
                taken_millis: 1000,
                review_kind: RevlogReviewKind::Review,
            };
            col.storage.add_revlog_entry(&entry, true).unwrap();
        }
        cid
    }

    fn id_base() -> i64 {
        TimestampSecs::now().0 * 1000
    }

    fn attempts(n: u32, correct: u32) -> Vec<u8> {
        (0..n).map(|i| if i < correct { 3 } else { 1 }).collect()
    }

    /// Seed `topics` with `attempts_each`/`correct_each` application reviews.
    fn seed_topics(col: &mut Collection, topics: &[&str], attempts_each: u32, correct_each: u32) {
        let app = add_application_notetype(col);
        let mut id = id_base();
        for tag in topics {
            seed_card(
                col,
                &app,
                tag,
                &attempts(attempts_each, correct_each),
                &mut id,
            );
        }
    }

    /// An eligible deck yields a whole-number score inside a valid 472-528
    /// band, formatted as points.
    #[test]
    fn eligible_deck_projects_onto_the_scale() {
        let mut col = Collection::new();
        seed_topics(&mut col, &[STRUCTURE, METABOLISM, FOLDING, KINETICS], 8, 6);

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
        // Only 3 attempts on 2 topics: Performance abstains → Readiness abstains.
        seed_topics(&mut col, &[STRUCTURE, KINETICS], 3, 2);

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
        seed_topics(&mut wide_col, &ALL_TOPICS, 8, 6);
        let full = wide_col.get_readiness_score(DeckId(1)).unwrap();

        // Half coverage, same per-topic record.
        let mut narrow_col = Collection::new();
        seed_topics(&mut narrow_col, &ALL_TOPICS[..4], 8, 6);
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
        seed_topics(&mut top, &ALL_TOPICS, 8, 8); // all correct, full coverage
        let top_score = top.get_readiness_score(DeckId(1)).unwrap();
        assert!(
            top_score.estimate >= 525.0,
            "perfect practice → near 528, got {}",
            top_score.estimate
        );

        let mut bottom = Collection::new();
        seed_topics(&mut bottom, &ALL_TOPICS, 8, 0); // all wrong, full coverage
        let bottom_score = bottom.get_readiness_score(DeckId(1)).unwrap();
        assert!(
            bottom_score.estimate <= 475.0,
            "no correct answers → near 472, got {}",
            bottom_score.estimate
        );
    }
}
