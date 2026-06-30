// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The honest **Memory score** — Speedrun Phase 2b, decisions D7, D8, D9
//! (see docs/plan/spec-scores.md, docs/plan/decisions.md).
//!
//! Memory answers "can I recall a taught fact now?" by aggregating FSRS
//! per-card retrievability over the deck's cards that map to in-scope AAMC
//! topics. It is rendered only as a whole [evidence envelope](MemoryScore)
//! (spec §4) — there is no bare-number path — and it is governed by a single
//! give-up rule (spec §5, D9): below a data floor it abstains and says what is
//! missing rather than guessing in a nice font.
//!
//! What it computes, over the in-scope cards (those whose note carries a
//! taxonomy-leaf tag, [`card_topic`]):
//!
//! - `estimate` = mean current FSRS retrievability over the *reviewed* in-scope
//!   cards. Retrievability comes from the shared [`card_retrievability`]; a card
//!   with no FSRS memory state contributes the same
//!   `NO_MEMORY_STATE_RETRIEVABILITY` (0.9) prior the queue uses (D23).
//! - `range` = the 95% interval of that mean (mean ± `Z_95`·SE over the per-card
//!   retrievabilities), clamped to `[0, 1]`. This is a *spread-based* interval,
//!   not yet model/calibration uncertainty — calibration is proven Sunday
//!   (spec §6, D8).
//! - `coverage_pct` = [`coverage_pct`] of in-scope topics that have ≥1 reviewed
//!   card.
//! - `graded_reviews` = scheduling-affecting revlog entries over the in-scope
//!   cards.
//! - `confidence` = low/medium/high from (`graded_reviews`, `coverage_pct`).
//! - `reasons` = the top drivers (coverage, the weakest covered topics).
//!
//! **Give-up rule (D9):** eligible ⇔ `graded_reviews ≥ MIN_GRADED_REVIEWS` AND
//! `coverage_pct ≥ MIN_COVERAGE_PCT`. When ineligible the score abstains
//! (`estimate`/`range` = 0) and `abstain_reason` names the failed condition(s)
//! and what clears them. All thresholds are named constants, flagged tunable
//! until real study histories exist (D9 gap).
//!
//! Read-only: like the topic queue, this only reads existing FSRS/revlog state
//! and never mutates the collection or touches scheduling. (Per-attempt
//! `AttemptLog` persistence from spec §8 is deferred — Wednesday's Memory score
//! needs only existing data; the reviewer logging hook is Phase 3.)

use std::collections::HashMap;

use fsrs::FSRS;

use crate::prelude::*;
use crate::search::SearchNode;
use crate::search::SortMode;
use crate::speedrun::card_signals::card_retrievability;
use crate::speedrun::card_signals::card_topic;
use crate::speedrun::card_signals::leaf_topic_weights;
use crate::speedrun::taxonomy::coverage_pct;
use crate::speedrun::taxonomy::seed_taxonomy;
use crate::timestamp::TimestampSecs;

/// Minimum scheduling-affecting reviews in scope before Memory will show a
/// number (D9). Tunable; a guess pending real study histories.
pub const MIN_GRADED_REVIEWS: u32 = 200;
/// Minimum fraction of in-scope topics that must be covered before Memory will
/// show a number (D9). Tunable.
pub const MIN_COVERAGE_PCT: f32 = 0.50;

/// `graded_reviews`/`coverage_pct` at or above these report "high" confidence.
const CONFIDENCE_HIGH_REVIEWS: u32 = 1_000;
const CONFIDENCE_HIGH_COVERAGE: f32 = 0.80;
/// …and these report "medium"; anything eligible but below is "low".
const CONFIDENCE_MEDIUM_REVIEWS: u32 = 500;
const CONFIDENCE_MEDIUM_COVERAGE: f32 = 0.65;

/// z for a 95% normal interval.
const Z_95: f32 = 1.96;
/// How many weak-topic drivers to surface in `reasons` (plus the coverage one).
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

/// The Memory evidence envelope (spec §4). The single rendered form of the
/// score; when `abstained`, `estimate`/`range_low`/`range_high` are 0 and
/// `abstain_reason` explains why.
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

/// Per-topic retrievability accumulator (running sum + count) for the weakest-
/// topic reasons.
#[derive(Default)]
struct TopicAccum {
    retrievability_sum: f32,
    card_count: u32,
}

impl Collection {
    /// Compute the Memory score for `deck_id` (and its children), spec §6.
    pub(crate) fn get_memory_score(&mut self, deck_id: DeckId) -> Result<MemoryScore> {
        let leaf_weights = leaf_topic_weights();
        let in_scope_topic_ids: Vec<String> = leaf_weights.keys().cloned().collect();

        let card_ids =
            self.search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)?;
        let timing = self.timing_today()?;
        let fsrs = FSRS::new(None)?;

        // One pass over the in-scope cards: tally reviews, collect the per-card
        // retrievabilities that feed the estimate, and the covered topics.
        let mut retrievabilities: Vec<f32> = Vec::new();
        let mut covered_topic_ids: Vec<String> = Vec::new();
        let mut graded_reviews: u32 = 0;
        let mut per_topic: HashMap<String, TopicAccum> = HashMap::new();
        for cid in card_ids {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            let Some(topic) = card_topic(&note.tags, &leaf_weights) else {
                continue;
            };
            let graded = self
                .storage
                .get_revlog_entries_for_card(cid)?
                .iter()
                .filter(|entry| entry.has_rating_and_affects_scheduling())
                .count() as u32;
            graded_reviews += graded;
            // A card counts toward the estimate and coverage only once it has
            // actually been graded — an unreviewed card has no recall evidence.
            if graded >= 1 {
                let retrievability = card_retrievability(&card, &timing, &fsrs);
                retrievabilities.push(retrievability);
                covered_topic_ids.push(topic.clone());
                let accum = per_topic.entry(topic).or_default();
                accum.retrievability_sum += retrievability;
                accum.card_count += 1;
            }
        }

        let coverage = coverage_pct(&in_scope_topic_ids, &covered_topic_ids);
        let (estimate, range_low, range_high) = mean_and_interval(&retrievabilities);
        let confidence = confidence_for(graded_reviews, coverage);
        let reasons = build_reasons(coverage, &per_topic);

        // Give-up rule (D9): show a number only with enough reviews AND coverage.
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

/// Mean of the values plus its 95% interval (mean ± `Z_95`·SE), clamped to
/// `[0, 1]`. Returns all-zero for an empty slice and a degenerate point
/// interval for a single value (only reachable when abstaining, where the
/// caller zeroes the estimate regardless).
fn mean_and_interval(values: &[f32]) -> (f32, f32, f32) {
    let n = values.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mean = values.iter().sum::<f32>() / n as f32;
    if n < 2 {
        return (mean, mean, mean);
    }
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / (n as f32 - 1.0);
    let std_err = (variance / n as f32).sqrt();
    let half = Z_95 * std_err;
    (mean, (mean - half).clamp(0.0, 1.0), (mean + half).clamp(0.0, 1.0))
}

fn confidence_for(graded_reviews: u32, coverage: f32) -> Confidence {
    if graded_reviews >= CONFIDENCE_HIGH_REVIEWS && coverage >= CONFIDENCE_HIGH_COVERAGE {
        Confidence::High
    } else if graded_reviews >= CONFIDENCE_MEDIUM_REVIEWS && coverage >= CONFIDENCE_MEDIUM_COVERAGE {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

/// Top drivers behind the score: coverage, then the weakest covered topics by
/// mean `1 - retrievability` (the memory signal), named by their taxonomy
/// label. Deterministic: ties break on topic id.
fn build_reasons(coverage: f32, per_topic: &HashMap<String, TopicAccum>) -> Vec<String> {
    let mut reasons = vec![format!("coverage {}%", pct_round(coverage))];

    let labels = topic_labels();
    let mut by_weakness: Vec<(&str, f32)> = per_topic
        .iter()
        .filter(|(_, accum)| accum.card_count > 0)
        .map(|(topic, accum)| {
            let mean_r = accum.retrievability_sum / accum.card_count as f32;
            (topic.as_str(), 1.0 - mean_r)
        })
        .collect();
    by_weakness.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    for (topic, _weakness) in by_weakness.into_iter().take(MAX_WEAK_TOPIC_REASONS) {
        let label = labels.get(topic).map(String::as_str).unwrap_or(topic);
        reasons.push(format!("weak: {label}"));
    }
    reasons
}

/// The failed give-up condition(s) and what clears them (spec §5).
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

/// In-scope leaf topic id → human label, for `reasons`.
fn topic_labels() -> HashMap<String, String> {
    seed_taxonomy()
        .into_iter()
        .filter(|node| node.in_scope)
        .map(|node| (node.id, node.label))
        .collect()
}

fn pct_round(fraction: f32) -> i64 {
    (fraction * 100.0).round() as i64
}

#[cfg(test)]
mod tests {
    use fsrs::FSRS5_DEFAULT_DECAY;

    use super::*;
    use crate::card::CardQueue;
    use crate::card::CardType;
    use crate::card::FsrsMemoryState;
    use crate::revlog::RevlogEntry;
    use crate::revlog::RevlogId;
    use crate::revlog::RevlogReviewKind;
    use crate::types::Usn;

    const STRUCTURE: &str = "mcat::biomolecules::amino_acids::structure";
    const PKA: &str = "mcat::biomolecules::amino_acids::pka_titration";
    const METABOLISM: &str = "mcat::biomolecules::amino_acids::metabolism";
    const FOLDING: &str = "mcat::biomolecules::proteins::folding";
    const KINETICS: &str = "mcat::biomolecules::enzymes::kinetics";

    /// Adds a review card due today, tagged `tag`, whose FSRS state sets its
    /// retrievability, then logs `reviews` scheduling-affecting revlog entries.
    /// `next_id` keeps revlog ids globally unique across cards.
    fn seed_card(
        col: &mut Collection,
        tag: &str,
        stability: f32,
        elapsed_days: i64,
        reviews: u32,
        next_id: &mut i64,
    ) -> CardId {
        let nt = col.basic_notetype();
        let mut note = nt.new_note();
        note.set_field(0, "front").unwrap();
        note.tags.push(tag.to_string());
        col.add_note(&mut note, DeckId(1)).unwrap();
        let cid = col.storage.card_ids_of_notes(&[note.id]).unwrap()[0];

        let mut card = col.storage.get_card(cid).unwrap().unwrap();
        card.ctype = CardType::Review;
        card.queue = CardQueue::Review;
        card.due = 0;
        card.interval = elapsed_days.max(1) as u32;
        card.memory_state = Some(FsrsMemoryState {
            stability,
            difficulty: 5.0,
        });
        card.decay = Some(FSRS5_DEFAULT_DECAY);
        card.last_review_time = Some(TimestampSecs::now().adding_secs(-elapsed_days * 86_400));
        col.storage.update_card(&card).unwrap();

        for _ in 0..reviews {
            *next_id += 1;
            let entry = RevlogEntry {
                id: RevlogId(*next_id),
                cid,
                usn: Usn(-1),
                button_chosen: 3,
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

    /// A base revlog id well above any timestamp the harness might generate.
    fn id_base() -> i64 {
        TimestampSecs::now().0 * 1000
    }

    /// Eligible deck (≥200 reviews, ≥50% coverage) returns a fully populated
    /// envelope with a real estimate and interval, not abstaining.
    #[test]
    fn eligible_deck_populates_full_envelope() {
        let mut col = Collection::new();
        let mut id = id_base();
        // Five of eight topics covered (62.5%), 50 graded reviews each = 250.
        for (tag, stability, elapsed) in [
            (STRUCTURE, 10.0, 20),
            (PKA, 30.0, 10),
            (METABOLISM, 5.0, 40),
            (FOLDING, 80.0, 5),
            (KINETICS, 50.0, 8),
        ] {
            seed_card(&mut col, tag, stability, elapsed, 50, &mut id);
        }

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(!score.abstained, "deck above the give-up line is eligible");
        assert!(score.abstain_reason.is_empty());
        assert_eq!(score.graded_reviews, 250);
        assert!((score.coverage_pct - 5.0 / 8.0).abs() < 1e-4, "5 of 8 topics");
        assert!(
            score.estimate > 0.0 && score.estimate <= 1.0,
            "real estimate in (0, 1], got {}",
            score.estimate
        );
        assert!(
            score.range_low <= score.estimate && score.estimate <= score.range_high,
            "estimate must sit inside its range"
        );
        assert!(score.range_low >= 0.0 && score.range_high <= 1.0, "range clamped");
        assert!(!score.reasons.is_empty(), "drivers are reported");
        assert!(score.updated_at_secs > 0);
    }

    /// Below 200 graded reviews (but well-covered), Memory abstains and the
    /// reason names the review shortfall; the number is hidden.
    #[test]
    fn abstains_below_review_threshold() {
        let mut col = Collection::new();
        let mut id = id_base();
        // Good coverage (5/8) but only 10 reviews each = 50 < 200.
        for tag in [STRUCTURE, PKA, METABOLISM, FOLDING, KINETICS] {
            seed_card(&mut col, tag, 20.0, 10, 10, &mut id);
        }

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 50);
        assert!(
            score.abstain_reason.contains("graded reviews"),
            "reason should name the review shortfall, got {:?}",
            score.abstain_reason
        );
        assert_eq!(score.estimate, 0.0, "no number when abstaining");
        assert_eq!(score.range_low, 0.0);
        assert_eq!(score.range_high, 0.0);
        // Coverage is still reported honestly even while abstaining.
        assert!((score.coverage_pct - 5.0 / 8.0).abs() < 1e-4);
    }

    /// Below 50% coverage (even with many reviews), Memory abstains and the
    /// reason names the coverage gap.
    #[test]
    fn abstains_below_coverage_threshold() {
        let mut col = Collection::new();
        let mut id = id_base();
        // Only 2/8 topics (25%) but 150 reviews each = 300 ≥ 200.
        for tag in [STRUCTURE, KINETICS] {
            seed_card(&mut col, tag, 20.0, 10, 150, &mut id);
        }

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 300);
        assert!(
            score.abstain_reason.contains("coverage"),
            "reason should name the coverage gap, got {:?}",
            score.abstain_reason
        );
        assert_eq!(score.estimate, 0.0);
        assert!((score.coverage_pct - 2.0 / 8.0).abs() < 1e-4);
    }

    /// The interval is a real distribution-based range, not a bare number:
    /// with varied per-card retrievability the low and high bounds differ.
    #[test]
    fn range_is_nondegenerate_when_eligible() {
        let mut col = Collection::new();
        let mut id = id_base();
        // Deliberately spread retrievability across the five covered topics.
        for (tag, stability, elapsed) in [
            (STRUCTURE, 2.0, 60),
            (PKA, 10.0, 30),
            (METABOLISM, 40.0, 15),
            (FOLDING, 120.0, 3),
            (KINETICS, 300.0, 1),
        ] {
            seed_card(&mut col, tag, stability, elapsed, 50, &mut id);
        }

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(!score.abstained);
        assert!(
            score.range_high - score.range_low > 1e-6,
            "range must be a real interval, got [{}, {}]",
            score.range_low,
            score.range_high
        );
    }

    /// `reasons` surfaces the weakest covered topic by name.
    #[test]
    fn weakest_topic_surfaces_in_reasons() {
        let mut col = Collection::new();
        let mut id = id_base();
        // KINETICS is by far the weakest (tiny stability, long elapsed); the
        // rest are strongly retained.
        seed_card(&mut col, KINETICS, 1.0, 90, 50, &mut id);
        for tag in [STRUCTURE, PKA, METABOLISM, FOLDING] {
            seed_card(&mut col, tag, 500.0, 1, 50, &mut id);
        }

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(!score.abstained);
        // "Kinetics" is the seed label for the KINETICS leaf.
        assert!(
            score.reasons.iter().any(|r| r == "weak: Kinetics"),
            "weakest topic should be named, got {:?}",
            score.reasons
        );
    }

    /// An empty deck has no evidence: abstain on both conditions, no number.
    #[test]
    fn empty_deck_abstains_on_both_conditions() {
        let mut col = Collection::new();

        let score = col.get_memory_score(DeckId(1)).unwrap();

        assert!(score.abstained);
        assert_eq!(score.graded_reviews, 0);
        assert_eq!(score.coverage_pct, 0.0);
        assert_eq!(score.estimate, 0.0);
        assert!(score.abstain_reason.contains("graded reviews"));
        assert!(score.abstain_reason.contains("coverage"));
    }
}
