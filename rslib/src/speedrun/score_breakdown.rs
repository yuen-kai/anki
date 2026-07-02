// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Per-leaf-topic **score breakdown** for the Speedrun study screen's
//! per-subject details (the "More details" modal).
//!
//! The Memory and Performance scores are whole-deck aggregates
//! ([`crate::speedrun::memory_score`], [`crate::speedrun::performance_score`]);
//! the modal needs their *inputs* split per topic so it can explain, subject by
//! subject, what feeds each score. This module does one read-only pass over the
//! deck's in-scope cards and reports, for every in-scope leaf topic:
//!
//! - `mean_retrievability` — mean FSRS retrievability over the topic's reviewed
//!   cards (the Memory input), the same per-topic mean as [`memory_score`].
//! - `application_accuracy` / `application_attempts` — accuracy over graded
//!   `SpeedrunApplication` attempts (the Performance input), the same tally as
//!   [`performance_score`] (Good-or-better = correct).
//! - `memory_reviews` — graded reviews over the topic's cards that feed Memory.
//! - `exam_weight` / `path` — the taxonomy weight and display path.
//! - `has_application_data` — whether the topic has any graded application
//!   attempt, so the UI can tell "0%" apart from "not practiced".
//!
//! Every in-scope leaf topic gets a row, even one with no cards (all zeros), so
//! the modal shows the full subject map including untouched topics. Read-only:
//! reads existing FSRS/revlog state and mutates nothing.
//!
//! [`memory_score`]: crate::speedrun::memory_score
//! [`performance_score`]: crate::speedrun::performance_score

use std::collections::HashMap;

use fsrs::FSRS;

use crate::prelude::*;
use crate::search::SearchNode;
use crate::search::SortMode;
use crate::speedrun::card_signals::card_retrievability;
use crate::speedrun::card_signals::card_topic;
use crate::speedrun::card_signals::leaf_topic_weights;
use crate::speedrun::progression::NoteKind;
use crate::speedrun::taxonomy::seed_taxonomy;
use crate::speedrun::taxonomy::topic_path_labels;

/// A rating of Good (3) or Easy (4) counts as getting an application item right
/// (same threshold as the Performance score).
const CORRECT_BUTTON: u8 = 3;

/// Per-leaf-topic score inputs for the study screen's per-subject breakdown.
#[derive(Debug, Clone)]
pub struct TopicStat {
    /// Taxonomy leaf id, e.g. `mcat::biomolecules::enzymes::kinetics`.
    pub topic_id: String,
    /// Display path foundation → leaf, e.g. `["Biomolecules", "Enzymes",
    /// "Kinetics"]`.
    pub path: Vec<String>,
    /// Mean FSRS retrievability over the topic's reviewed cards (Memory input);
    /// 0 when the topic has no reviewed card.
    pub mean_retrievability: f32,
    /// Accuracy over graded application attempts (Performance input); 0 when
    /// the topic has no application attempt.
    pub application_accuracy: f32,
    /// Graded application attempts backing `application_accuracy`.
    pub application_attempts: u32,
    /// Graded reviews over the topic's cards that feed the Memory score.
    pub memory_reviews: u32,
    /// The topic's share of exam emphasis.
    pub exam_weight: f32,
    /// Whether the topic has at least one graded application attempt.
    pub has_application_data: bool,
}

/// Running per-topic tally over the deck's cards.
#[derive(Default)]
struct Accum {
    retrievability_sum: f32,
    reviewed_cards: u32,
    memory_reviews: u32,
    application_attempts: u32,
    application_correct: u32,
}

impl Collection {
    /// Per-leaf-topic score inputs for `deck_id` (and its children), for the
    /// study screen's per-subject details. One read-only pass over the in-scope
    /// cards; every in-scope leaf topic is returned (untouched ones as zeros).
    pub(crate) fn get_speedrun_score_breakdown(
        &mut self,
        deck_id: DeckId,
    ) -> Result<Vec<TopicStat>> {
        let leaf_weights = leaf_topic_weights();
        let card_ids =
            self.search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)?;
        let timing = self.timing_today()?;
        let fsrs = FSRS::new(None)?;

        // One pass over the in-scope cards: retrievability + graded reviews feed
        // Memory; application cards additionally feed the accuracy tally, so the
        // two inputs stay separate exactly as the whole-deck scores keep them.
        let mut per_topic: HashMap<String, Accum> = HashMap::new();
        for cid in card_ids {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            let Some(topic) = card_topic(&note.tags, &leaf_weights) else {
                continue;
            };
            let is_application = self.note_kind(&note)? == NoteKind::Application;

            let mut graded = 0u32;
            let mut correct = 0u32;
            for entry in self.storage.get_revlog_entries_for_card(cid)? {
                if entry.has_rating_and_affects_scheduling() {
                    graded += 1;
                    if entry.button_chosen >= CORRECT_BUTTON {
                        correct += 1;
                    }
                }
            }

            let accum = per_topic.entry(topic).or_default();
            accum.memory_reviews += graded;
            // A card feeds the retrievability mean only once graded — an
            // unreviewed card carries no recall evidence (matches memory_score).
            if graded >= 1 {
                accum.retrievability_sum += card_retrievability(&card, &timing, &fsrs);
                accum.reviewed_cards += 1;
            }
            if is_application {
                accum.application_attempts += graded;
                accum.application_correct += correct;
            }
        }

        // One row per in-scope leaf topic, in the taxonomy's display order, so
        // untouched topics still appear (as zeros) and the ordering is stable.
        let stats = seed_taxonomy()
            .into_iter()
            .filter(|node| node.in_scope)
            .map(|node| {
                let accum = per_topic.get(&node.id);
                let mean_retrievability = accum
                    .filter(|a| a.reviewed_cards > 0)
                    .map(|a| a.retrievability_sum / a.reviewed_cards as f32)
                    .unwrap_or(0.0);
                let application_attempts = accum.map(|a| a.application_attempts).unwrap_or(0);
                let application_correct = accum.map(|a| a.application_correct).unwrap_or(0);
                let application_accuracy = if application_attempts > 0 {
                    application_correct as f32 / application_attempts as f32
                } else {
                    0.0
                };
                TopicStat {
                    path: topic_path_labels(&node.id),
                    mean_retrievability,
                    application_accuracy,
                    application_attempts,
                    memory_reviews: accum.map(|a| a.memory_reviews).unwrap_or(0),
                    exam_weight: node.exam_weight,
                    has_application_data: application_attempts > 0,
                    topic_id: node.id,
                }
            })
            .collect();
        Ok(stats)
    }
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
    use crate::timestamp::TimestampSecs;
    use crate::types::Usn;

    const STRUCTURE: &str = "mcat::biomolecules::amino_acids::structure"; // weight 0.15
    const KINETICS: &str = "mcat::biomolecules::enzymes::kinetics"; // weight 0.18
    const FOLDING: &str = "mcat::biomolecules::proteins::folding"; // weight 0.10

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

    /// A base revlog id well above any timestamp the harness might generate.
    fn id_base() -> i64 {
        TimestampSecs::now().0 * 1000
    }

    /// Add an application card tagged `tag`, logging one graded review per
    /// button (1 Again … 4 Easy). `next_id` keeps revlog ids unique across
    /// cards.
    fn add_application_card(
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

    /// Add a reviewed (basic) card with an FSRS memory state, for the Memory
    /// input; logs `reviews` scheduling-affecting revlog entries.
    fn add_memory_card(
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

    /// `n` graded attempts, `correct` of them Good — as a button vector.
    fn attempts(n: u32, correct: u32) -> Vec<u8> {
        (0..n).map(|i| if i < correct { 3 } else { 1 }).collect()
    }

    /// Every in-scope leaf topic is reported, with the Memory and Performance
    /// inputs computed independently per topic.
    #[test]
    fn breakdown_reports_every_in_scope_topic_with_per_topic_inputs() {
        let mut col = Collection::new();
        let app = add_notetype_named(&mut col, "SpeedrunApplication");
        let mut id = id_base();
        // STRUCTURE: 10 application attempts, 7 correct → accuracy 0.7.
        add_application_card(&mut col, &app, STRUCTURE, &attempts(10, 7), &mut id);
        // KINETICS: a well-retained memory card, no application data.
        add_memory_card(&mut col, KINETICS, 100.0, 1, 5, &mut id);

        let stats = col.get_speedrun_score_breakdown(DeckId(1)).unwrap();

        // The seed taxonomy has eight in-scope leaf topics; all appear.
        assert_eq!(stats.len(), 8);

        let structure = stats.iter().find(|s| s.topic_id == STRUCTURE).unwrap();
        assert_eq!(structure.application_attempts, 10);
        assert!((structure.application_accuracy - 0.7).abs() < 1e-4);
        assert!(structure.has_application_data);
        // Application reviews also feed Memory (they are in-scope cards).
        assert_eq!(structure.memory_reviews, 10);
        assert_eq!(
            structure.path,
            vec!["Biomolecules", "Amino Acids", "Structure"]
        );
        assert!((structure.exam_weight - 0.15).abs() < 1e-6);

        let kinetics = stats.iter().find(|s| s.topic_id == KINETICS).unwrap();
        assert!(!kinetics.has_application_data);
        assert_eq!(kinetics.application_attempts, 0);
        assert_eq!(kinetics.memory_reviews, 5);
        assert!(
            kinetics.mean_retrievability > 0.5,
            "a recent high-stability card is well retained, got {}",
            kinetics.mean_retrievability
        );

        // A topic with no cards is still present, all zeros.
        let folding = stats.iter().find(|s| s.topic_id == FOLDING).unwrap();
        assert_eq!(folding.memory_reviews, 0);
        assert_eq!(folding.application_attempts, 0);
        assert!(!folding.has_application_data);
        assert_eq!(folding.mean_retrievability, 0.0);
    }

    /// A memory-only (basic) card must not create application data, keeping the
    /// Performance input honest even when the topic has Memory evidence.
    #[test]
    fn breakdown_keeps_application_data_isolated_from_memory_only_cards() {
        let mut col = Collection::new();
        let mut id = id_base();
        add_memory_card(&mut col, STRUCTURE, 50.0, 5, 8, &mut id);

        let stats = col.get_speedrun_score_breakdown(DeckId(1)).unwrap();
        let structure = stats.iter().find(|s| s.topic_id == STRUCTURE).unwrap();

        assert_eq!(structure.memory_reviews, 8);
        assert!(
            !structure.has_application_data,
            "a basic card is not application data"
        );
        assert_eq!(structure.application_attempts, 0);
        assert_eq!(structure.application_accuracy, 0.0);
    }
}
