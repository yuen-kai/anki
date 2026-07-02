// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Performance score** for the authored study engine.
//!
//! Performance answers "can I work an application problem right?" from the
//! student's accuracy on the authored concepts' application-stage attempts. The
//! `speedrun-review` screen records those attempts per concept
//! ([`crate::speedrun::study`]): grading a concept at Applying/Mastering
//! (`hierarchy`/`mastering`) — the stages that render an application problem —
//! increments its `appTotal`, and a ≥Good rating its `appCorrect`. So the two
//! are the same engine: applying problems on the study screen moves this score.
//!
//! It is deliberately a *different* construct from Memory (recall of a taught
//! concept via FSRS), so the two are never conflated: Memory reads
//! retrievability off the concept cards, Performance reads the application
//! attempt tally.
//!
//! What it computes, over the concepts with ≥1 application attempt:
//!
//! - `estimate` = exam-weight-weighted mean of per-concept accuracy
//!   (`appCorrect / appTotal`), weighted like Memory: a concept's authored
//!   leaf's AAMC weight when its title matches a taxonomy label
//!   (case-insensitive), else the taxonomy's mean leaf weight.
//! - `range` = the 95% interval of the pooled accuracy, centered on `estimate`.
//! - `coverage_pct` = fraction of the deck's concepts with an application
//!   attempt.
//! - `graded_reviews` = total application attempts in the deck.
//! - `confidence` = low/medium/high from (attempts, coverage).
//! - `reasons` = coverage plus the weakest applied leaves.
//!
//! **Give-up rule:** eligible ⇔ `graded_reviews ≥ PERF_MIN_GRADED_ATTEMPTS` AND
//! `coverage_pct ≥ PERF_MIN_COVERAGE_PCT`; else it abstains and names the
//! shortfall. Thresholds are tunable, sized for authored decks.
//!
//! Read-only: reads the authored progress store + hierarchy; mutates nothing.

use std::collections::HashMap;

use crate::prelude::*;
use crate::speedrun::scores::confidence_from;
use crate::speedrun::scores::pct_round;
use crate::speedrun::scores::ScoreEnvelope;
use crate::speedrun::scores::ScoreFormat;
use crate::speedrun::taxonomy::leaf_weight_by_label;
use crate::speedrun::taxonomy::mean_leaf_weight;
use crate::timestamp::TimestampSecs;

/// Minimum application attempts before Performance shows a number. Tunable.
pub const PERF_MIN_GRADED_ATTEMPTS: u32 = 5;
/// Minimum fraction of the deck's concepts with an application attempt before
/// Performance shows a number. Tunable.
pub const PERF_MIN_COVERAGE_PCT: f32 = 0.25;

/// `attempts`/`coverage` at or above these report "high" confidence.
const CONFIDENCE_HIGH_ATTEMPTS: u32 = 40;
const CONFIDENCE_HIGH_COVERAGE: f32 = 0.80;
/// …and these report "medium"; anything eligible but below is "low".
const CONFIDENCE_MEDIUM_ATTEMPTS: u32 = 15;
const CONFIDENCE_MEDIUM_COVERAGE: f32 = 0.50;

/// How many weak-leaf drivers to surface in `reasons` (plus the coverage one).
const MAX_WEAK_TOPIC_REASONS: usize = 2;

/// A concept's application accuracy contribution.
struct AppliedConcept {
    correct: u32,
    attempts: u32,
    weight: f32,
    leaf_title: String,
}

impl Collection {
    /// Compute the Performance score for `deck_id` from the authored concepts'
    /// application attempts.
    pub(crate) fn get_performance_score(&mut self, deck_id: DeckId) -> Result<ScoreEnvelope> {
        let leaves = self.speedrun_authored_leaves(deck_id);
        let progress = self.speedrun_deck_progress(deck_id);
        let weights_by_label = leaf_weight_by_label();
        let neutral_weight = mean_leaf_weight();

        let total_concepts: usize = leaves.iter().map(|l| l.concept_ids.len()).sum();
        let mut applied: Vec<AppliedConcept> = Vec::new();
        let mut total_attempts: u32 = 0;
        let mut total_correct: u32 = 0;
        for leaf in &leaves {
            let weight = weights_by_label
                .get(&leaf.title.to_lowercase())
                .copied()
                .unwrap_or(neutral_weight);
            for concept_id in &leaf.concept_ids {
                let Some(entry) = progress.get(concept_id) else {
                    continue;
                };
                if entry.app_total == 0 {
                    continue;
                }
                total_attempts += entry.app_total;
                total_correct += entry.app_correct;
                applied.push(AppliedConcept {
                    correct: entry.app_correct,
                    attempts: entry.app_total,
                    weight,
                    leaf_title: leaf.title.clone(),
                });
            }
        }

        let coverage = if total_concepts == 0 {
            0.0
        } else {
            applied.len() as f32 / total_concepts as f32
        };
        let estimate = weighted_accuracy(&applied);
        let pooled = if total_attempts > 0 {
            total_correct as f32 / total_attempts as f32
        } else {
            0.0
        };
        let (range_low, range_high) =
            ScoreEnvelope::proportion_interval(estimate, pooled, total_attempts);
        let confidence = confidence_from(
            total_attempts,
            coverage,
            CONFIDENCE_HIGH_ATTEMPTS,
            CONFIDENCE_HIGH_COVERAGE,
            CONFIDENCE_MEDIUM_ATTEMPTS,
            CONFIDENCE_MEDIUM_COVERAGE,
        );
        let reasons = build_reasons(coverage, &applied);

        let eligible =
            total_attempts >= PERF_MIN_GRADED_ATTEMPTS && coverage >= PERF_MIN_COVERAGE_PCT;
        if !eligible {
            return Ok(ScoreEnvelope::abstained(
                coverage,
                total_attempts,
                abstain_reason(total_attempts, coverage),
                ScoreFormat::Ratio,
            ));
        }

        Ok(ScoreEnvelope {
            estimate,
            range_low,
            range_high,
            coverage_pct: coverage,
            confidence,
            updated_at_secs: TimestampSecs::now().0,
            reasons,
            abstained: false,
            abstain_reason: String::new(),
            graded_reviews: total_attempts,
            format: ScoreFormat::Ratio,
        })
    }
}

/// Exam-weight-weighted mean of per-concept accuracy; falls back to the pooled
/// (unweighted) accuracy if no applied concept carries weight.
fn weighted_accuracy(applied: &[AppliedConcept]) -> f32 {
    let mut weight_sum = 0.0;
    let mut weighted = 0.0;
    for concept in applied {
        if concept.attempts == 0 {
            continue;
        }
        weight_sum += concept.weight;
        weighted += concept.weight * (concept.correct as f32 / concept.attempts as f32);
    }
    if weight_sum > 0.0 {
        return weighted / weight_sum;
    }
    let attempts: u32 = applied.iter().map(|c| c.attempts).sum();
    let correct: u32 = applied.iter().map(|c| c.correct).sum();
    if attempts > 0 {
        correct as f32 / attempts as f32
    } else {
        0.0
    }
}

/// Top drivers: coverage, then the weakest applied leaves by `1 - accuracy`,
/// named by the authored leaf title. Deterministic: ties break on title.
fn build_reasons(coverage: f32, applied: &[AppliedConcept]) -> Vec<String> {
    let mut reasons = vec![format!("coverage {}%", pct_round(coverage))];

    let mut sums: HashMap<&str, (u32, u32)> = HashMap::new();
    for concept in applied {
        let entry = sums.entry(concept.leaf_title.as_str()).or_insert((0, 0));
        entry.0 += concept.correct;
        entry.1 += concept.attempts;
    }
    let mut by_weakness: Vec<(&str, f32)> = sums
        .iter()
        .filter(|(_, (_, attempts))| *attempts > 0)
        .map(|(title, (correct, attempts))| (*title, 1.0 - *correct as f32 / *attempts as f32))
        .collect();
    by_weakness.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    for (title, _weakness) in by_weakness.into_iter().take(MAX_WEAK_TOPIC_REASONS) {
        reasons.push(format!("weak: {title}"));
    }
    reasons
}

/// The failed give-up condition(s) and what clears them.
fn abstain_reason(attempts: u32, coverage: f32) -> String {
    let mut failed = Vec::new();
    if attempts < PERF_MIN_GRADED_ATTEMPTS {
        failed.push(format!(
            "needs {PERF_MIN_GRADED_ATTEMPTS} application attempts, have {attempts}"
        ));
    }
    if coverage < PERF_MIN_COVERAGE_PCT {
        failed.push(format!(
            "coverage {}% below the {}% minimum",
            pct_round(coverage),
            pct_round(PERF_MIN_COVERAGE_PCT)
        ));
    }
    failed.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speedrun::study::testing::apply_correct_attempts;
    use crate::speedrun::study::testing::grade_concept;
    use crate::speedrun::study::testing::learn_leaf;
    use crate::speedrun::study::testing::set_hierarchy;

    /// Studying (applying problems) through the real answer path moves
    /// Performance from abstaining to a real number — the acceptance-style
    /// proof that the authored flow feeds the score.
    #[test]
    fn applying_stage_study_moves_performance() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[("Kinetics", &["k1"]), ("Inhibition", &["i1"])],
        );

        // Nothing applied yet: no attempts, abstains.
        let before = col.get_performance_score(DeckId(1)).unwrap();
        assert!(before.abstained);
        assert_eq!(before.graded_reviews, 0);

        // Apply each concept three times (Good never leaves the application
        // stage, so all six are correct application attempts).
        apply_correct_attempts(&mut col, DeckId(1), &["k1"], "k1", 3);
        apply_correct_attempts(&mut col, DeckId(1), &["i1"], "i1", 3);

        let after = col.get_performance_score(DeckId(1)).unwrap();
        assert!(!after.abstained, "deck above the give-up line is eligible");
        assert_eq!(after.graded_reviews, 6);
        assert!((after.coverage_pct - 1.0).abs() < 1e-4, "2 of 2 concepts");
        assert!(
            after.estimate > 0.9,
            "all correct → near 1.0, got {}",
            after.estimate
        );
        assert_eq!(after.format.as_str(), "ratio");
    }

    /// Below the attempt floor Performance abstains and names the shortfall.
    #[test]
    fn abstains_below_attempt_threshold() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["k1", "k2", "k3", "k4"])]);
        // Only two application attempts (2 < 5).
        col.speedrun_seed_app_counts(DeckId(1), "k1", 2, 2);

        let score = col.get_performance_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert!(
            score.abstain_reason.contains("application attempts"),
            "reason names the attempt shortfall, got {:?}",
            score.abstain_reason
        );
        assert_eq!(score.estimate, 0.0);
    }

    /// Practicing/learning grades are recall, not application, so they never
    /// feed Performance: a deck only ever practiced abstains with 0 attempts.
    #[test]
    fn practicing_grades_are_not_application_attempts() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["k1"])]);
        learn_leaf(&mut col, DeckId(1), &["k1"]);
        // One Good at practicing: recall, not an application attempt.
        grade_concept(&mut col, DeckId(1), "k1", 3);

        let score = col.get_performance_score(DeckId(1)).unwrap();
        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 0, "no application attempts recorded");
    }

    /// A wrong application attempt lowers accuracy without abstaining once the
    /// evidence floor is cleared.
    #[test]
    fn wrong_attempts_lower_accuracy() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[("Kinetics", &["k1"]), ("Inhibition", &["i1"])],
        );
        // k1: 4 attempts, 2 correct. i1: 4 attempts, 4 correct.
        col.speedrun_seed_app_counts(DeckId(1), "k1", 2, 4);
        col.speedrun_seed_app_counts(DeckId(1), "i1", 4, 4);

        let score = col.get_performance_score(DeckId(1)).unwrap();
        assert!(!score.abstained);
        assert_eq!(score.graded_reviews, 8);
        assert!(
            score.estimate > 0.0 && score.estimate < 1.0,
            "mixed record → strictly between 0 and 1, got {}",
            score.estimate
        );
    }

    /// Exam weight pulls the estimate toward heavier topics: the heavy topic
    /// answered right and the light one wrong lands above the plain 50%.
    #[test]
    fn exam_weight_pulls_estimate_toward_heavier_topics() {
        let mut col = Collection::new();
        // Kinetics (0.18) all right, Metabolism (0.08) all wrong, equal attempts.
        set_hierarchy(
            &mut col,
            "1",
            &[("Kinetics", &["k1"]), ("Metabolism", &["m1"])],
        );
        col.speedrun_seed_app_counts(DeckId(1), "k1", 6, 6);
        col.speedrun_seed_app_counts(DeckId(1), "m1", 0, 6);

        let score = col.get_performance_score(DeckId(1)).unwrap();
        assert!(!score.abstained);
        assert!(
            score.estimate > 0.55,
            "weighted toward the heavier correct topic (>0.5), got {}",
            score.estimate
        );
    }
}
