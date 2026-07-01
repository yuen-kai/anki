// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Performance score** (spec-scores §7, decision D33).
//!
//! Performance answers "can I get a *new, exam-style* question right?" from the
//! student's real accuracy on `SpeedrunApplication` cards (the application
//! problems), weighted by exam weight, rendered as the shared evidence envelope
//! and governed by the give-up rule. It is deliberately a *different* construct
//! from Memory (recall of a taught fact via FSRS), so the two are never
//! conflated (D7): Memory reads retrievability, Performance reads graded
//! application attempts.
//!
//! What it computes, over the in-scope `SpeedrunApplication` cards (note carries
//! a taxonomy-leaf tag, [`card_topic`]):
//!
//! - `estimate` = exam-weight-weighted mean of per-topic accuracy, where a
//!   topic's accuracy is `correct / attempts` over its graded reviews and
//!   `correct` means a rating of Good or better (button ≥ 3).
//! - `range` = the 95% interval of the pooled accuracy, centered on `estimate`.
//! - `coverage_pct` = in-scope topics with ≥1 graded application attempt.
//! - `graded_reviews` = total graded application attempts in scope.
//! - `confidence` = low/medium/high from (attempts, coverage).
//! - `reasons` = coverage, the weakest application topics, and the honest note
//!   that it measures *practiced* items (the unseen-item paraphrase test is the
//!   Friday milestone).
//!
//! **Give-up rule (D9/D33):** eligible ⇔ `graded_reviews ≥
//! PERF_MIN_GRADED_ATTEMPTS` AND `coverage_pct ≥ PERF_MIN_COVERAGE_PCT`; else it
//! abstains and names the shortfall. Thresholds are tunable guesses pending real
//! study histories.
//!
//! Read-only: reads existing revlog + note/card state; mutates nothing.

use std::collections::HashMap;

use crate::prelude::*;
use crate::search::SearchNode;
use crate::search::SortMode;
use crate::speedrun::card_signals::card_topic;
use crate::speedrun::card_signals::leaf_topic_weights;
use crate::speedrun::progression::NoteKind;
use crate::speedrun::scores::confidence_from;
use crate::speedrun::scores::pct_round;
use crate::speedrun::scores::topic_labels;
use crate::speedrun::scores::ScoreEnvelope;
use crate::speedrun::scores::ScoreFormat;
use crate::speedrun::taxonomy::coverage_pct;
use crate::timestamp::TimestampSecs;

/// Minimum graded application attempts in scope before Performance shows a
/// number (D33). Lower than Memory's review floor because application items are
/// scarcer and each carries more signal. Tunable.
pub const PERF_MIN_GRADED_ATTEMPTS: u32 = 30;
/// Minimum fraction of in-scope topics with an application attempt before
/// Performance shows a number (D33). Tunable.
pub const PERF_MIN_COVERAGE_PCT: f32 = 0.50;

/// `attempts`/`coverage` at or above these report "high" confidence.
const CONFIDENCE_HIGH_ATTEMPTS: u32 = 200;
const CONFIDENCE_HIGH_COVERAGE: f32 = 0.80;
/// …and these report "medium"; anything eligible but below is "low".
const CONFIDENCE_MEDIUM_ATTEMPTS: u32 = 80;
const CONFIDENCE_MEDIUM_COVERAGE: f32 = 0.65;

/// How many weak-topic drivers to surface in `reasons` (plus the coverage one).
const MAX_WEAK_TOPIC_REASONS: usize = 2;

/// A rating of Good (3) or Easy (4) counts as getting the item right.
const CORRECT_BUTTON: u8 = 3;

/// Per-topic correct/attempts accumulator over application reviews.
#[derive(Default)]
struct TopicAccuracy {
    correct: u32,
    attempts: u32,
}

impl Collection {
    /// Compute the Performance score for `deck_id` (and its children), spec §7.
    pub(crate) fn get_performance_score(&mut self, deck_id: DeckId) -> Result<ScoreEnvelope> {
        let leaf_weights = leaf_topic_weights();
        let in_scope_topic_ids: Vec<String> = leaf_weights.keys().cloned().collect();

        let card_ids =
            self.search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)?;

        // One pass: over in-scope SpeedrunApplication cards, tally correct/total
        // graded attempts per topic. Non-application cards never contribute, so
        // Performance can't collapse into Memory.
        let mut per_topic: HashMap<String, TopicAccuracy> = HashMap::new();
        let mut total_attempts: u32 = 0;
        let mut total_correct: u32 = 0;
        for cid in card_ids {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            if self.note_kind(&note)? != NoteKind::Application {
                continue;
            }
            let Some(topic) = card_topic(&note.tags, &leaf_weights) else {
                continue;
            };
            let mut attempts = 0u32;
            let mut correct = 0u32;
            for entry in self.storage.get_revlog_entries_for_card(cid)? {
                if entry.has_rating_and_affects_scheduling() {
                    attempts += 1;
                    if entry.button_chosen >= CORRECT_BUTTON {
                        correct += 1;
                    }
                }
            }
            if attempts == 0 {
                continue;
            }
            total_attempts += attempts;
            total_correct += correct;
            let accum = per_topic.entry(topic).or_default();
            accum.attempts += attempts;
            accum.correct += correct;
        }

        let covered_topic_ids: Vec<String> = per_topic.keys().cloned().collect();
        let coverage = coverage_pct(&in_scope_topic_ids, &covered_topic_ids);

        let estimate = weighted_accuracy(&per_topic, &leaf_weights);
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
        let reasons = build_reasons(coverage, &per_topic);

        // Give-up rule (D33): a number only with enough attempts AND coverage.
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

/// Exam-weight-weighted mean of per-topic accuracy over covered topics; falls
/// back to the pooled (unweighted) accuracy if no covered topic carries weight.
fn weighted_accuracy(
    per_topic: &HashMap<String, TopicAccuracy>,
    weights: &HashMap<String, f32>,
) -> f32 {
    let mut weight_sum = 0.0;
    let mut weighted = 0.0;
    for (topic, accum) in per_topic {
        if accum.attempts == 0 {
            continue;
        }
        let weight = weights.get(topic).copied().unwrap_or(0.0);
        weight_sum += weight;
        weighted += weight * (accum.correct as f32 / accum.attempts as f32);
    }
    if weight_sum > 0.0 {
        return weighted / weight_sum;
    }
    let attempts: u32 = per_topic.values().map(|a| a.attempts).sum();
    let correct: u32 = per_topic.values().map(|a| a.correct).sum();
    if attempts > 0 {
        correct as f32 / attempts as f32
    } else {
        0.0
    }
}

/// Top drivers: coverage, the weakest application topics by `1 - accuracy`, and
/// the honest scope note (practiced items, paraphrase test is Friday).
fn build_reasons(coverage: f32, per_topic: &HashMap<String, TopicAccuracy>) -> Vec<String> {
    let mut reasons = vec![format!("coverage {}%", pct_round(coverage))];

    let labels = topic_labels();
    let mut by_weakness: Vec<(&str, f32)> = per_topic
        .iter()
        .filter(|(_, accum)| accum.attempts > 0)
        .map(|(topic, accum)| {
            (
                topic.as_str(),
                1.0 - accum.correct as f32 / accum.attempts as f32,
            )
        })
        .collect();
    by_weakness.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    for (topic, _weakness) in by_weakness.into_iter().take(MAX_WEAK_TOPIC_REASONS) {
        let label = labels.get(topic).map(String::as_str).unwrap_or(topic);
        reasons.push(format!("weak: {label}"));
    }

    reasons.push(
        "measures practiced application items; unseen-item validation (paraphrase test) is \
         the Friday milestone"
            .to_string(),
    );
    reasons
}

/// The failed give-up condition(s) and what clears them (spec §5).
fn abstain_reason(attempts: u32, coverage: f32) -> String {
    let mut failed = Vec::new();
    if attempts < PERF_MIN_GRADED_ATTEMPTS {
        failed.push(format!(
            "needs {PERF_MIN_GRADED_ATTEMPTS} graded application attempts, have {attempts}"
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
    use crate::revlog::RevlogEntry;
    use crate::revlog::RevlogId;
    use crate::revlog::RevlogReviewKind;
    use crate::types::Usn;

    const STRUCTURE: &str = "mcat::biomolecules::amino_acids::structure"; // 0.15
    const METABOLISM: &str = "mcat::biomolecules::amino_acids::metabolism"; // 0.08
    const FOLDING: &str = "mcat::biomolecules::proteins::folding"; // 0.10
    const KINETICS: &str = "mcat::biomolecules::enzymes::kinetics"; // 0.18

    fn add_notetype_named(col: &mut Collection, name: &str) -> Notetype {
        let mut nt = Notetype {
            name: name.to_string(),
            ..Default::default()
        };
        nt.add_field("Front");
        nt.add_template("Card 1", "{{Front}}", "{{Front}}");
        col.add_notetype(&mut nt, true).unwrap();
        nt
    }

    /// Add a card of `nt` tagged `tag`, then log one graded review per button in
    /// `buttons` (1 Again … 4 Easy). `next_id` keeps revlog ids unique.
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

    /// `n` graded attempts, `correct` of them Good — as a button vector.
    fn attempts(n: u32, correct: u32) -> Vec<u8> {
        (0..n)
            .map(|i| if i < correct { 3 } else { 1 })
            .collect()
    }

    /// Eligible deck (≥30 application attempts, ≥50% coverage) returns a full
    /// envelope with a real estimate inside its range, not abstaining.
    #[test]
    fn eligible_deck_populates_full_envelope() {
        let mut col = Collection::new();
        let app = add_notetype_named(&mut col, "SpeedrunApplication");
        let mut id = id_base();
        // 4 of 8 topics (50%), 8 attempts each (32), 6 correct each → acc 0.75.
        for tag in [STRUCTURE, METABOLISM, FOLDING, KINETICS] {
            seed_card(&mut col, &app, tag, &attempts(8, 6), &mut id);
        }

        let score = col.get_performance_score(DeckId(1)).unwrap();

        assert!(!score.abstained, "deck above the give-up line is eligible");
        assert!(score.abstain_reason.is_empty());
        assert_eq!(score.graded_reviews, 32);
        assert!((score.coverage_pct - 0.5).abs() < 1e-4, "4 of 8 topics");
        assert!(
            score.estimate > 0.0 && score.estimate <= 1.0,
            "real estimate, got {}",
            score.estimate
        );
        assert!(
            score.range_low <= score.estimate && score.estimate <= score.range_high,
            "estimate inside its range"
        );
        assert_eq!(score.format.as_str(), "ratio");
        assert!(!score.reasons.is_empty());
    }

    /// Below the attempt floor (but well-covered), Performance abstains and the
    /// reason names the application-attempt shortfall.
    #[test]
    fn abstains_below_attempt_threshold() {
        let mut col = Collection::new();
        let app = add_notetype_named(&mut col, "SpeedrunApplication");
        let mut id = id_base();
        // 4/8 topics but only 3 attempts each = 12 < 30.
        for tag in [STRUCTURE, METABOLISM, FOLDING, KINETICS] {
            seed_card(&mut col, &app, tag, &attempts(3, 2), &mut id);
        }

        let score = col.get_performance_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 12);
        assert!(
            score.abstain_reason.contains("application attempts"),
            "reason names the attempt shortfall, got {:?}",
            score.abstain_reason
        );
        assert_eq!(score.estimate, 0.0, "no number when abstaining");
        assert_eq!(score.format.as_str(), "ratio");
    }

    /// Below 50% coverage (even with many attempts), Performance abstains and the
    /// reason names the coverage gap.
    #[test]
    fn abstains_below_coverage_threshold() {
        let mut col = Collection::new();
        let app = add_notetype_named(&mut col, "SpeedrunApplication");
        let mut id = id_base();
        // Only 2/8 topics (25%) but 20 attempts each = 40 ≥ 30.
        for tag in [STRUCTURE, KINETICS] {
            seed_card(&mut col, &app, tag, &attempts(20, 15), &mut id);
        }

        let score = col.get_performance_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 40);
        assert!(
            score.abstain_reason.contains("coverage"),
            "reason names the coverage gap, got {:?}",
            score.abstain_reason
        );
    }

    /// Exam weight matters: with the heavier topics answered right and the
    /// lighter ones wrong, the weighted estimate sits above the plain 50%.
    #[test]
    fn exam_weight_pulls_estimate_toward_heavier_topics() {
        let mut col = Collection::new();
        let app = add_notetype_named(&mut col, "SpeedrunApplication");
        let mut id = id_base();
        // Heavier topics all correct, lighter all wrong; equal attempts each.
        for tag in [KINETICS, STRUCTURE] {
            seed_card(&mut col, &app, tag, &attempts(8, 8), &mut id);
        }
        for tag in [FOLDING, METABOLISM] {
            seed_card(&mut col, &app, tag, &attempts(8, 0), &mut id);
        }

        let score = col.get_performance_score(DeckId(1)).unwrap();

        assert!(!score.abstained);
        assert!(
            score.estimate > 0.55,
            "weighted toward the heavier correct topics (>0.5), got {}",
            score.estimate
        );
        assert!(score.estimate < 1.0);
    }

    /// Only application cards count: a Basic card and a SpeedrunConcept card,
    /// both reviewed, never enter the Performance scope.
    #[test]
    fn non_application_cards_are_excluded() {
        let mut col = Collection::new();
        let app = add_notetype_named(&mut col, "SpeedrunApplication");
        let concept = add_notetype_named(&mut col, "SpeedrunConcept");
        let mut id = id_base();
        for tag in [STRUCTURE, METABOLISM, FOLDING, KINETICS] {
            seed_card(&mut col, &app, tag, &attempts(8, 6), &mut id);
        }
        // These must not change graded_reviews (32) or the estimate.
        seed_card(&mut col, &concept, KINETICS, &attempts(10, 0), &mut id);
        let basic = col.basic_notetype();
        seed_card(&mut col, &basic, STRUCTURE, &attempts(10, 0), &mut id);

        let score = col.get_performance_score(DeckId(1)).unwrap();

        assert_eq!(
            score.graded_reviews, 32,
            "only SpeedrunApplication attempts are counted"
        );
    }

    /// An eligible deck answered entirely wrong is *not* abstaining (the data is
    /// there) — it honestly scores near zero.
    #[test]
    fn all_wrong_eligible_scores_near_zero() {
        let mut col = Collection::new();
        let app = add_notetype_named(&mut col, "SpeedrunApplication");
        let mut id = id_base();
        for tag in [STRUCTURE, METABOLISM, FOLDING, KINETICS] {
            seed_card(&mut col, &app, tag, &attempts(8, 0), &mut id);
        }

        let score = col.get_performance_score(DeckId(1)).unwrap();

        assert!(!score.abstained, "enough data → not abstaining");
        assert_eq!(score.graded_reviews, 32);
        assert!(score.estimate < 1e-4, "all wrong → ~0, got {}", score.estimate);
    }
}
