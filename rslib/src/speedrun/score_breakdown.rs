// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Per-authored-leaf **score breakdown** for the study screen's per-subject
//! details (the "More details" modal).
//!
//! The Memory and Performance scores are whole-deck aggregates
//! ([`crate::speedrun::memory_score`], [`crate::speedrun::performance_score`]);
//! the modal needs their *inputs* split per authored leaf topic so it can
//! explain, subject by subject, what feeds each score. This module does one
//! read-only pass over the deck's authored leaves and reports, for each:
//!
//! - `mean_retrievability` — mean FSRS retrievability over the leaf's graded
//!   concept cards (the Memory input).
//! - `application_accuracy` / `application_attempts` — accuracy over the leaf's
//!   concepts' application attempts (the Performance input).
//! - `memory_reviews` — graded reviews over the leaf's concept cards.
//! - `exam_weight` / `path` — the mapped AAMC weight (0 when the leaf title
//!   matches no taxonomy label) and the authored display path.
//! - `has_application_data` — whether the leaf has any application attempt, so
//!   the UI can tell "0%" apart from "not practiced".
//!
//! Every authored leaf gets a row, even an untouched one (all zeros).
//! Read-only.

use crate::prelude::*;
use crate::speedrun::taxonomy::leaf_weight_by_label;

/// Per-authored-leaf score inputs for the study screen's per-subject breakdown.
#[derive(Debug, Clone)]
pub struct TopicStat {
    /// Authored leaf node id.
    pub topic_id: String,
    /// Display path, e.g. `["Enzymes", "Kinetics"]`.
    pub path: Vec<String>,
    /// Mean FSRS retrievability over the leaf's graded concept cards (Memory
    /// input); 0 when the leaf has no graded concept.
    pub mean_retrievability: f32,
    /// Accuracy over the leaf's application attempts (Performance input); 0
    /// when the leaf has no application attempt.
    pub application_accuracy: f32,
    /// Application attempts backing `application_accuracy`.
    pub application_attempts: u32,
    /// Graded reviews over the leaf's concept cards that feed the Memory score.
    pub memory_reviews: u32,
    /// The leaf's AAMC exam weight (0 when its title matches no taxonomy
    /// label).
    pub exam_weight: f32,
    /// Whether the leaf has at least one application attempt.
    pub has_application_data: bool,
}

impl Collection {
    /// Per-authored-leaf score inputs for `deck_id`, for the study screen's
    /// per-subject details. One read-only pass over the authored leaves; every
    /// leaf is returned (untouched ones as zeros).
    pub(crate) fn get_speedrun_score_breakdown(
        &mut self,
        deck_id: DeckId,
    ) -> Result<Vec<TopicStat>> {
        let leaves = self.speedrun_authored_leaves(deck_id);
        let stats = self.speedrun_concept_card_stats(deck_id)?;
        let progress = self.speedrun_deck_progress(deck_id);
        let weights_by_label = leaf_weight_by_label();

        let mut out = Vec::with_capacity(leaves.len());
        for leaf in leaves {
            let mut retrievability_sum = 0.0;
            let mut graded_concepts = 0u32;
            let mut memory_reviews = 0u32;
            let mut application_attempts = 0u32;
            let mut application_correct = 0u32;
            for concept_id in &leaf.concept_ids {
                if let Some(stat) = stats.get(concept_id) {
                    memory_reviews += stat.graded_reviews;
                    if stat.graded_reviews >= 1 {
                        retrievability_sum += stat.retrievability;
                        graded_concepts += 1;
                    }
                }
                if let Some(entry) = progress.get(concept_id) {
                    application_attempts += entry.app_total;
                    application_correct += entry.app_correct;
                }
            }
            let mean_retrievability = if graded_concepts > 0 {
                retrievability_sum / graded_concepts as f32
            } else {
                0.0
            };
            let application_accuracy = if application_attempts > 0 {
                application_correct as f32 / application_attempts as f32
            } else {
                0.0
            };
            out.push(TopicStat {
                topic_id: leaf.node_id,
                path: leaf.path,
                mean_retrievability,
                application_accuracy,
                application_attempts,
                memory_reviews,
                exam_weight: weights_by_label
                    .get(&leaf.title.to_lowercase())
                    .copied()
                    .unwrap_or(0.0),
                has_application_data: application_attempts > 0,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speedrun::study::testing::grade_concept;
    use crate::speedrun::study::testing::set_hierarchy;

    /// Every authored leaf is reported, with the Memory and Performance inputs
    /// computed independently per leaf.
    #[test]
    fn breakdown_reports_every_leaf_with_per_leaf_inputs() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[("Kinetics", &["k1"]), ("Folding", &["f1"])],
        );
        // Kinetics: a graded memory review plus seeded application evidence.
        grade_concept(&mut col, DeckId(1), "k1", 3);
        col.speedrun_seed_app_counts(DeckId(1), "k1", 3, 4);
        // Folding: only a memory review, no application.
        grade_concept(&mut col, DeckId(1), "f1", 3);

        let stats = col.get_speedrun_score_breakdown(DeckId(1)).unwrap();
        assert_eq!(stats.len(), 2, "one row per authored leaf");

        let kinetics = stats
            .iter()
            .find(|s| s.path.last().unwrap() == "Kinetics")
            .unwrap();
        assert_eq!(kinetics.application_attempts, 4);
        assert!((kinetics.application_accuracy - 0.75).abs() < 1e-4);
        assert!(kinetics.has_application_data);
        assert!(kinetics.memory_reviews >= 1);
        // "Kinetics" matches a taxonomy label, so it carries its exam weight.
        assert!((kinetics.exam_weight - 0.18).abs() < 1e-6);

        let folding = stats
            .iter()
            .find(|s| s.path.last().unwrap() == "Folding")
            .unwrap();
        assert!(!folding.has_application_data);
        assert_eq!(folding.application_attempts, 0);
        assert_eq!(folding.memory_reviews, 1);
        assert!(folding.mean_retrievability > 0.0);
    }

    /// An untouched authored leaf still appears, all zeros.
    #[test]
    fn untouched_leaf_is_all_zeros() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[("Kinetics", &["k1"]), ("Regulation", &["r1"])],
        );
        grade_concept(&mut col, DeckId(1), "k1", 3);

        let stats = col.get_speedrun_score_breakdown(DeckId(1)).unwrap();
        let regulation = stats
            .iter()
            .find(|s| s.path.last().unwrap() == "Regulation")
            .unwrap();
        assert_eq!(regulation.memory_reviews, 0);
        assert_eq!(regulation.application_attempts, 0);
        assert!(!regulation.has_application_data);
        assert_eq!(regulation.mean_retrievability, 0.0);
    }
}
