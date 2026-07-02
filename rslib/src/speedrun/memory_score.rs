// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Memory score** for the authored study engine.
//!
//! Memory answers "can I recall a taught concept now?" by aggregating FSRS
//! per-card retrievability over the deck's authored concepts. Each authored
//! concept is materialized into exactly one `SpeedrunItem` card
//! ([`crate::speedrun::study`]), so studying the `speedrun-review` screen
//! grades those cards and moves this score — the two are the same engine.
//!
//! It is rendered only as a whole [evidence envelope](MemoryScore) — there is
//! no bare-number path — and it is governed by a single give-up rule: below a
//! data floor it abstains and says what is missing rather than guessing.
//!
//! What it computes, over the concepts whose card has ≥1 graded review:
//!
//! - `estimate` = the exam-weight-weighted mean of per-concept retrievability.
//!   A concept's weight is its authored leaf's AAMC exam weight when the leaf
//!   title matches a taxonomy label (case-insensitive), else the taxonomy's
//!   mean leaf weight (a neutral share), so mapped and unmapped concepts stay
//!   comparable. This is the hierarchy roll-up: a leaf's memory is the mean
//!   over its concepts, and the deck's is the weighted mean over leaves.
//! - `range` = the 95% interval of that mean (estimate ± `Z_95`·SE over the
//!   per-concept retrievabilities), a spread-based interval, clamped to `[0,
//!   1]`.
//! - `coverage_pct` = fraction of the deck's concepts with ≥1 graded review.
//! - `graded_reviews` = scheduling-affecting revlog entries over the concept
//!   cards.
//! - `confidence` = low/medium/high from (`graded_reviews`, `coverage_pct`).
//! - `reasons` = coverage plus the weakest covered leaves.
//!
//! **Give-up rule:** eligible ⇔ `graded_reviews ≥ MIN_GRADED_REVIEWS` AND
//! `coverage_pct ≥ MIN_COVERAGE_PCT`. When ineligible it abstains
//! (`estimate`/`range` = 0) and `abstain_reason` names the failed condition(s).
//! Thresholds are named constants, tunable and sized for authored decks (one
//! card per concept, not thousands of tagged cards).
//!
//! Read-only: reads the materialized cards' FSRS/revlog state and the authored
//! hierarchy; mutates nothing.

use std::collections::HashMap;

use crate::prelude::*;
use crate::speedrun::scores::confidence_from;
use crate::speedrun::scores::pct_round;
use crate::speedrun::scores::Z_95;
use crate::speedrun::taxonomy::leaf_weight_by_label;
use crate::speedrun::taxonomy::mean_leaf_weight;
use crate::timestamp::TimestampSecs;

/// Minimum graded concept reviews before Memory shows a number. Tunable; sized
/// for authored decks where each concept is a single card.
pub const MIN_GRADED_REVIEWS: u32 = 5;
/// Minimum fraction of the deck's concepts that must be graded before Memory
/// shows a number. Tunable.
pub const MIN_COVERAGE_PCT: f32 = 0.25;

/// `graded_reviews`/`coverage_pct` at or above these report "high" confidence.
const CONFIDENCE_HIGH_REVIEWS: u32 = 40;
const CONFIDENCE_HIGH_COVERAGE: f32 = 0.80;
/// …and these report "medium"; anything eligible but below is "low".
const CONFIDENCE_MEDIUM_REVIEWS: u32 = 15;
const CONFIDENCE_MEDIUM_COVERAGE: f32 = 0.50;

/// How many weak-leaf drivers to surface in `reasons` (plus the coverage one).
const MAX_WEAK_TOPIC_REASONS: usize = 2;

/// Confidence band reported alongside the estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        }
    }
}

/// The Memory evidence envelope. The single rendered form of the score; when
/// `abstained`, `estimate`/`range_low`/`range_high` are 0 and `abstain_reason`
/// explains why.
#[derive(Debug, Clone)]
pub struct MemoryScore {
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
}

/// A concept's contribution to the Memory aggregate.
struct GradedConcept {
    retrievability: f32,
    weight: f32,
    leaf_title: String,
}

impl Collection {
    /// Compute the Memory score for `deck_id` from its authored concepts.
    pub(crate) fn get_memory_score(&mut self, deck_id: DeckId) -> Result<MemoryScore> {
        let leaves = self.speedrun_authored_leaves(deck_id);
        let stats = self.speedrun_concept_card_stats(deck_id)?;
        let weights_by_label = leaf_weight_by_label();
        let neutral_weight = mean_leaf_weight();

        let total_concepts: usize = leaves.iter().map(|l| l.concept_ids.len()).sum();
        let mut graded: Vec<GradedConcept> = Vec::new();
        let mut graded_reviews: u32 = 0;
        for leaf in &leaves {
            let weight = weights_by_label
                .get(&leaf.title.to_lowercase())
                .copied()
                .unwrap_or(neutral_weight);
            for concept_id in &leaf.concept_ids {
                let Some(stat) = stats.get(concept_id) else {
                    continue;
                };
                graded_reviews += stat.graded_reviews;
                if stat.graded_reviews >= 1 {
                    graded.push(GradedConcept {
                        retrievability: stat.retrievability,
                        weight,
                        leaf_title: leaf.title.clone(),
                    });
                }
            }
        }

        let coverage = if total_concepts == 0 {
            0.0
        } else {
            graded.len() as f32 / total_concepts as f32
        };
        let (estimate, range_low, range_high) = weighted_mean_and_interval(&graded);
        let confidence = confidence_from(
            graded_reviews,
            coverage,
            CONFIDENCE_HIGH_REVIEWS,
            CONFIDENCE_HIGH_COVERAGE,
            CONFIDENCE_MEDIUM_REVIEWS,
            CONFIDENCE_MEDIUM_COVERAGE,
        );
        let reasons = build_reasons(coverage, &graded);

        let eligible = graded_reviews >= MIN_GRADED_REVIEWS && coverage >= MIN_COVERAGE_PCT;
        let (estimate, range_low, range_high, abstain_reason) = if eligible {
            (estimate, range_low, range_high, String::new())
        } else {
            (0.0, 0.0, 0.0, abstain_reason(graded_reviews, coverage))
        };

        Ok(MemoryScore {
            estimate,
            range_low,
            range_high,
            coverage_pct: coverage,
            confidence,
            updated_at_secs: TimestampSecs::now().0,
            reasons,
            abstained: !eligible,
            abstain_reason,
            graded_reviews,
        })
    }
}

/// The exam-weight-weighted mean of the graded concepts' retrievability, plus
/// its 95% interval (mean ± `Z_95`·SE over the unweighted per-concept spread),
/// clamped to `[0, 1]`. All-zero for an empty slice; a degenerate point
/// interval for a single concept (only reachable while abstaining).
fn weighted_mean_and_interval(concepts: &[GradedConcept]) -> (f32, f32, f32) {
    let n = concepts.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let weight_sum: f32 = concepts.iter().map(|c| c.weight).sum();
    let mean = if weight_sum > 0.0 {
        concepts
            .iter()
            .map(|c| c.weight * c.retrievability)
            .sum::<f32>()
            / weight_sum
    } else {
        concepts.iter().map(|c| c.retrievability).sum::<f32>() / n as f32
    };
    if n < 2 {
        return (mean, mean, mean);
    }
    // Spread of the raw per-concept retrievabilities around their plain mean.
    let plain_mean = concepts.iter().map(|c| c.retrievability).sum::<f32>() / n as f32;
    let variance = concepts
        .iter()
        .map(|c| (c.retrievability - plain_mean).powi(2))
        .sum::<f32>()
        / (n as f32 - 1.0);
    let half = Z_95 * (variance / n as f32).sqrt();
    (
        mean,
        (mean - half).clamp(0.0, 1.0),
        (mean + half).clamp(0.0, 1.0),
    )
}

/// Top drivers: coverage, then the weakest covered leaves by mean
/// `1 - retrievability`, named by the authored leaf title. Deterministic: ties
/// break on title.
fn build_reasons(coverage: f32, concepts: &[GradedConcept]) -> Vec<String> {
    let mut reasons = vec![format!("coverage {}%", pct_round(coverage))];

    let mut sums: HashMap<&str, (f32, u32)> = HashMap::new();
    for c in concepts {
        let entry = sums.entry(c.leaf_title.as_str()).or_insert((0.0, 0));
        entry.0 += c.retrievability;
        entry.1 += 1;
    }
    let mut by_weakness: Vec<(&str, f32)> = sums
        .iter()
        .map(|(title, (sum, count))| (*title, 1.0 - sum / *count as f32))
        .collect();
    by_weakness.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    for (title, _weakness) in by_weakness.into_iter().take(MAX_WEAK_TOPIC_REASONS) {
        reasons.push(format!("weak: {title}"));
    }
    reasons
}

/// The failed give-up condition(s) and what clears them.
fn abstain_reason(graded_reviews: u32, coverage: f32) -> String {
    let mut failed = Vec::new();
    if graded_reviews < MIN_GRADED_REVIEWS {
        failed.push(format!(
            "needs {MIN_GRADED_REVIEWS} graded reviews, have {graded_reviews}"
        ));
    }
    if coverage < MIN_COVERAGE_PCT {
        failed.push(format!(
            "coverage {}% below the {}% minimum",
            pct_round(coverage),
            pct_round(MIN_COVERAGE_PCT)
        ));
    }
    failed.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speedrun::study::testing::grade_concept;
    use crate::speedrun::study::testing::set_hierarchy;

    /// A deck with enough graded concepts is eligible and returns a real
    /// estimate inside its clamped range, not abstaining.
    #[test]
    fn eligible_deck_populates_full_envelope() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[
                ("Kinetics", &["k1", "k2"]),
                ("Inhibition", &["i1", "i2"]),
                ("Folding", &["f1", "f2"]),
            ],
        );
        // Grade every concept twice (12 graded reviews, full coverage).
        for cid in ["k1", "k2", "i1", "i2", "f1", "f2"] {
            grade_concept(&mut col, DeckId(1), cid, 3);
            grade_concept(&mut col, DeckId(1), cid, 3);
        }

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(!score.abstained, "deck above the give-up line is eligible");
        assert!(score.abstain_reason.is_empty());
        assert_eq!(score.graded_reviews, 12);
        assert!((score.coverage_pct - 1.0).abs() < 1e-4, "6 of 6 concepts");
        assert!(
            score.estimate > 0.0 && score.estimate <= 1.0,
            "real estimate in (0, 1], got {}",
            score.estimate
        );
        assert!(
            score.range_low <= score.estimate && score.estimate <= score.range_high,
            "estimate must sit inside its range"
        );
        assert!(!score.reasons.is_empty(), "drivers are reported");
    }

    /// Below the review floor Memory abstains and names the shortfall.
    #[test]
    fn abstains_below_review_threshold() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["k1", "k2", "k3", "k4"])]);
        // Only two graded concepts (2 < 5).
        grade_concept(&mut col, DeckId(1), "k1", 3);
        grade_concept(&mut col, DeckId(1), "k2", 3);

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 2);
        assert!(
            score.abstain_reason.contains("graded reviews"),
            "reason names the review shortfall, got {:?}",
            score.abstain_reason
        );
        assert_eq!(score.estimate, 0.0, "no number when abstaining");
    }

    /// An empty (unauthored) deck has no evidence: abstain, no number.
    #[test]
    fn empty_deck_abstains() {
        let mut col = Collection::new();
        let score = col.get_memory_score(DeckId(1)).unwrap();
        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 0);
        assert_eq!(score.coverage_pct, 0.0);
        assert_eq!(score.estimate, 0.0);
    }

    /// Studying more concepts raises coverage (the acceptance-style check that
    /// the authored study flow actually moves Memory).
    #[test]
    fn studying_more_concepts_raises_coverage() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[(
                "Kinetics",
                &["k1", "k2", "k3", "k4", "k5", "k6", "k7", "k8"],
            )],
        );
        for cid in ["k1", "k2", "k3"] {
            grade_concept(&mut col, DeckId(1), cid, 3);
        }
        let before = col.get_memory_score(DeckId(1)).unwrap();
        for cid in ["k4", "k5", "k6", "k7", "k8"] {
            grade_concept(&mut col, DeckId(1), cid, 3);
        }
        let after = col.get_memory_score(DeckId(1)).unwrap();

        assert!(
            after.coverage_pct > before.coverage_pct,
            "coverage should rise as more concepts are studied ({} -> {})",
            before.coverage_pct,
            after.coverage_pct
        );
        assert!(after.graded_reviews > before.graded_reviews);
    }

    /// The materialized-card helper keys cards by concept id.
    #[test]
    fn concept_card_stats_map_to_concepts() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["k1", "k2"])]);
        col.speedrun_reconcile(DeckId(1)).unwrap();
        let stats = col.speedrun_concept_card_stats(DeckId(1)).unwrap();
        assert_eq!(stats.len(), 2);
        assert!(stats.contains_key("k1") && stats.contains_key("k2"));
        // Ungraded card: no reviews yet, retrievability is the no-memory prior.
        assert_eq!(stats["k1"].graded_reviews, 0);
    }
}
