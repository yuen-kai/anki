// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Topic-grouped ("Learn") review queue — Speedrun Phase 2a, decisions D3,
//! D16, D17 (see docs/plan/decisions.md).
//!
//! An *additive* alternative ordering of today's due review cards: instead of
//! the default interleaved queue, cards are regrouped into topic-contiguous
//! blocks, the blocks ordered by `topic_weakness × exam_weight` and the cards
//! within a block by weakness (lowest FSRS retrievability first).
//!
//! It is read-only queue construction. Gathering, limits, the [`QueuedCards`]
//! return shape and the downstream `answer_card` path are all reused verbatim,
//! so this code path never produces or alters an FSRS interval and never
//! touches the undo queue (spec §8). The only thing it decides is the order in
//! which already-due cards are presented.
//!
//! ## Weakness proxy (Wednesday)
//!
//! Weakness is the Phase-1 [`topic_weakness`] blend of miss rate and
//! forgetting, fed two per-topic signals derived here:
//!
//! - `mean_retrievability`: the mean FSRS current retrievability over the
//!   topic's due cards (same computation as the stats graphs). A card with no
//!   FSRS memory state (e.g. an SM-2 card, or one moved with "set due date")
//!   contributes the shared `NO_MEMORY_STATE_RETRIEVABILITY` prior, a mild
//!   "probably still known" value, so such cards never masquerade as the
//!   weakest. Retrievability and the card→topic mapping come from
//!   [`crate::speedrun::card_signals`], shared with the Memory score.
//! - `recent_accuracy`: the pass rate (button ≥ Hard) over the topic's most
//!   recent [`RECENT_REVIEW_WINDOW`] graded reviews. With no graded history we
//!   fall back to `mean_retrievability`, collapsing the blend onto the memory
//!   signal rather than inventing an accuracy number.
//!
//! Both are deliberately simple and fully documented; the Friday/Sunday plan
//! swaps them for the Performance model's signal (spec §11).

use std::collections::HashMap;

use fsrs::FSRS;

use super::new_scheduling_context;
use super::QueueEntryKind;
use super::QueuedCard;
use super::QueuedCards;
use crate::card::Card;
use crate::prelude::*;
use crate::speedrun::card_signals::card_retrievability;
use crate::speedrun::card_signals::card_topic;
use crate::speedrun::card_signals::leaf_topic_weights;
use crate::speedrun::taxonomy::topic_weakness;

/// How many of a topic's most recent graded reviews feed `recent_accuracy`.
const RECENT_REVIEW_WINDOW: usize = 50;

/// A gathered due review card paired with the two values the ordering needs:
/// its topic (if any) and its current FSRS retrievability.
struct ReviewCardData {
    card: Card,
    /// The taxonomy leaf id this card maps to, or `None` (unmapped).
    topic: Option<String>,
    /// FSRS current retrievability in `[0, 1]`; weakness is `1 - this`.
    retrievability: f32,
}

impl Collection {
    /// Build the topic-grouped review queue for `deck_id` (spec §4–§5).
    ///
    /// Reuses the standard builder for gathering + deck/limit handling, keeps
    /// only the due review cards, then regroups them. `fetch_limit` caps the
    /// returned cards (`0` = no limit); the reported `review_count` is always
    /// the full block total, mirroring [`Collection::get_queued_cards`].
    pub(crate) fn get_topic_grouped_queue(
        &mut self,
        deck_id: DeckId,
        fetch_limit: usize,
    ) -> Result<QueuedCards> {
        // build_queues() returns an *owned* queue and does not replace the live
        // study queue, so the default Practice path is left untouched.
        let review_ids: Vec<CardId> = {
            let queues = self.build_queues(deck_id)?;
            queues
                .iter()
                .filter(|entry| matches!(entry.kind(), QueueEntryKind::Review))
                .map(|entry| entry.card_id())
                .collect()
        };
        let review_count = review_ids.len();

        let timing = self.timing_today()?;
        let fsrs = FSRS::new(None)?;
        let leaf_weights = leaf_topic_weights();

        // Pass 1: resolve each card's topic and retrievability.
        let mut data: Vec<ReviewCardData> = Vec::with_capacity(review_count);
        for cid in review_ids {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            // A card's topic is the note tag equal to a taxonomy leaf id; if
            // several match, the smallest id keeps the choice deterministic.
            let topic = card_topic(&note.tags, &leaf_weights);
            let retrievability = card_retrievability(&card, &timing, &fsrs);
            data.push(ReviewCardData {
                card,
                topic,
                retrievability,
            });
        }

        // Pass 2: per-topic recent accuracy from the revlog.
        let mut topics: Vec<String> = data.iter().filter_map(|d| d.topic.clone()).collect();
        topics.sort();
        topics.dedup();
        let mut topic_accuracy: HashMap<String, Option<f32>> = HashMap::new();
        for topic in &topics {
            let cids: Vec<CardId> = data
                .iter()
                .filter(|d| d.topic.as_deref() == Some(topic.as_str()))
                .map(|d| d.card.id)
                .collect();
            topic_accuracy.insert(topic.clone(), self.topic_recent_accuracy(&cids)?);
        }

        let ordered = order_review_cards(data, &leaf_weights, &topic_accuracy);

        // Assemble the standard QueuedCards payload (states + context), exactly
        // as the default queue does, so the reviewer/answer path is reused.
        let limit = if fetch_limit == 0 {
            ordered.len()
        } else {
            fetch_limit.min(ordered.len())
        };
        let mut cards = Vec::with_capacity(limit);
        for card in ordered.into_iter().take(limit) {
            let states = self.get_scheduling_states(card.id)?;
            let context = new_scheduling_context(self, &card)?;
            cards.push(QueuedCard {
                card,
                kind: QueueEntryKind::Review,
                states,
                context,
            });
        }

        Ok(QueuedCards {
            cards,
            new_count: 0,
            learning_count: 0,
            review_count,
        })
    }

    /// Pass rate over a topic's most recent graded reviews, or `None` when the
    /// topic has no graded history yet.
    fn topic_recent_accuracy(&self, card_ids: &[CardId]) -> Result<Option<f32>> {
        let mut entries = Vec::new();
        for cid in card_ids {
            for entry in self.storage.get_revlog_entries_for_card(*cid)? {
                if entry.has_rating_and_affects_scheduling() {
                    entries.push(entry);
                }
            }
        }
        if entries.is_empty() {
            return Ok(None);
        }
        // Most recent first (revlog id is a millisecond timestamp), capped.
        entries.sort_by(|a, b| b.id.0.cmp(&a.id.0));
        entries.truncate(RECENT_REVIEW_WINDOW);
        // Again (1) is a miss; Hard/Good/Easy (≥2) are passes.
        let passes = entries.iter().filter(|e| e.button_chosen >= 2).count();
        Ok(Some(passes as f32 / entries.len() as f32))
    }
}

/// Mean retrievability over a non-empty block; `0.0` for an empty slice.
fn mean_retrievability(cards: &[ReviewCardData]) -> f32 {
    if cards.is_empty() {
        return 0.0;
    }
    cards.iter().map(|c| c.retrievability).sum::<f32>() / cards.len() as f32
}

/// Order cards into topic-contiguous blocks by `block_priority` (descending),
/// cards within a block by weakness (lowest retrievability first), with the
/// unmapped block trailing. Deterministic: ties break on topic id then card id.
fn order_review_cards(
    cards: Vec<ReviewCardData>,
    leaf_weights: &HashMap<String, f32>,
    topic_accuracy: &HashMap<String, Option<f32>>,
) -> Vec<Card> {
    let mut blocks: HashMap<String, Vec<ReviewCardData>> = HashMap::new();
    let mut unmapped: Vec<ReviewCardData> = Vec::new();
    for card in cards {
        match &card.topic {
            Some(topic) => blocks.entry(topic.clone()).or_default().push(card),
            None => unmapped.push(card),
        }
    }

    // block_priority = topic_weakness(recent_accuracy, mean_retrievability)
    //                  × exam_weight   (spec §5)
    let mut block_list: Vec<(String, f32, Vec<ReviewCardData>)> = blocks
        .into_iter()
        .map(|(topic, cards)| {
            let mean_r = mean_retrievability(&cards);
            let accuracy = topic_accuracy
                .get(&topic)
                .copied()
                .flatten()
                .unwrap_or(mean_r);
            let weight = leaf_weights.get(&topic).copied().unwrap_or(0.0);
            let priority = topic_weakness(accuracy, mean_r) * weight;
            (topic, priority, cards)
        })
        .collect();
    block_list.sort_by(|(topic_a, prio_a, _), (topic_b, prio_b, _)| {
        prio_b.total_cmp(prio_a).then_with(|| topic_a.cmp(topic_b))
    });

    let mut ordered: Vec<Card> = Vec::new();
    for (_topic, _priority, mut cards) in block_list {
        sort_block_by_weakness(&mut cards);
        ordered.extend(cards.into_iter().map(|c| c.card));
    }
    sort_block_by_weakness(&mut unmapped);
    ordered.extend(unmapped.into_iter().map(|c| c.card));
    ordered
}

/// Sort a block so the weakest cards (lowest retrievability) come first, ties
/// broken by ascending card id.
fn sort_block_by_weakness(cards: &mut [ReviewCardData]) {
    cards.sort_by(|a, b| {
        a.retrievability
            .total_cmp(&b.retrievability)
            .then_with(|| a.card.id.cmp(&b.card.id))
    });
}

#[cfg(test)]
mod tests {
    use fsrs::FSRS5_DEFAULT_DECAY;

    use super::*;
    use crate::card::CardQueue;
    use crate::card::CardType;
    use crate::card::FsrsMemoryState;
    use crate::scheduler::answering::CardAnswer;
    use crate::scheduler::answering::Rating;
    use crate::scheduler::states::SchedulingStates;
    use crate::timestamp::TimestampMillis;
    use crate::timestamp::TimestampSecs;

    const KINETICS: &str = "mcat::biomolecules::enzymes::kinetics"; // weight 0.18
    const STRUCTURE: &str = "mcat::biomolecules::amino_acids::structure"; // weight 0.15
    const PKA: &str = "mcat::biomolecules::amino_acids::pka_titration"; // weight 0.12
    const METABOLISM: &str = "mcat::biomolecules::amino_acids::metabolism"; // weight 0.08

    /// Adds a review card due today, tagged `tag`, with an FSRS memory state
    /// whose `stability`/`elapsed_days` set its current retrievability.
    fn add_review_card(
        col: &mut Collection,
        tag: Option<&str>,
        stability: f32,
        elapsed_days: i64,
    ) -> CardId {
        let nt = col.basic_notetype();
        let mut note = nt.new_note();
        note.set_field(0, "front").unwrap();
        if let Some(tag) = tag {
            note.tags.push(tag.to_string());
        }
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
        cid
    }

    /// The topic each returned card maps to, in queue order, for assertions.
    fn queue_topics(col: &mut Collection, of: &HashMap<CardId, &str>) -> Vec<String> {
        let queued = col.get_topic_grouped_queue(DeckId(1), 0).unwrap();
        queued
            .cards
            .iter()
            .map(|qc| of[&qc.card.id].to_string())
            .collect()
    }

    /// (1) Cards from interleaved topics come back grouped into contiguous
    /// blocks — each topic occupies a single run, never reappearing later.
    #[test]
    fn cards_group_into_topic_contiguous_blocks() {
        let mut col = Collection::new();
        let mut of: HashMap<CardId, &str> = HashMap::new();
        // Interleave creation order across three topics.
        for _ in 0..3 {
            of.insert(add_review_card(&mut col, Some(KINETICS), 10.0, 5), KINETICS);
            of.insert(
                add_review_card(&mut col, Some(STRUCTURE), 10.0, 5),
                STRUCTURE,
            );
            of.insert(add_review_card(&mut col, Some(PKA), 10.0, 5), PKA);
        }

        let topics = queue_topics(&mut col, &of);
        assert_eq!(topics.len(), 9);

        // Each distinct topic must appear as exactly one contiguous run.
        let mut runs: Vec<&String> = Vec::new();
        for topic in &topics {
            if runs.last().map(|t| *t != topic).unwrap_or(true) {
                runs.push(topic);
            }
        }
        let mut seen = runs.clone();
        seen.sort();
        seen.dedup();
        assert_eq!(
            runs.len(),
            seen.len(),
            "each topic should form a single contiguous block, got {topics:?}"
        );
    }

    /// (2) Blocks are ordered by block_priority = weakness × exam_weight:
    /// equal weakness ranks by weight, and a high-weight but strong topic
    /// still sinks below weak ones.
    #[test]
    fn block_order_follows_block_priority() {
        let mut col = Collection::new();
        let mut of: HashMap<CardId, &str> = HashMap::new();
        // PKA and METABOLISM are equally weak (same stability/elapsed) but PKA
        // carries more weight (0.12 vs 0.08). KINETICS has the highest weight
        // (0.18) yet is strongly retained, so its weakness ≈ 0.
        of.insert(add_review_card(&mut col, Some(PKA), 1.0, 60), PKA);
        of.insert(
            add_review_card(&mut col, Some(METABOLISM), 1.0, 60),
            METABOLISM,
        );
        of.insert(
            add_review_card(&mut col, Some(KINETICS), 100_000.0, 1),
            KINETICS,
        );

        let topics = queue_topics(&mut col, &of);
        assert_eq!(
            topics,
            vec![
                PKA.to_string(),
                METABOLISM.to_string(),
                KINETICS.to_string()
            ],
            "weak-and-heavier first, strong-but-heaviest last"
        );
    }

    /// (3) Cards with no taxonomy tag form a single block that trails every
    /// mapped topic.
    #[test]
    fn unmapped_cards_trail_mapped_blocks() {
        let mut col = Collection::new();
        let mut of: HashMap<CardId, &str> = HashMap::new();
        of.insert(add_review_card(&mut col, Some(KINETICS), 1.0, 60), KINETICS);
        // A non-taxonomy tag and a tagless card both count as unmapped.
        of.insert(
            add_review_card(&mut col, Some("speedrun::seed"), 1.0, 60),
            "",
        );
        of.insert(add_review_card(&mut col, None, 1.0, 60), "");

        let topics = queue_topics(&mut col, &of);
        assert_eq!(topics.len(), 3);
        assert_eq!(topics[0], KINETICS, "the mapped block comes first");
        assert_eq!(&topics[1..], &["".to_string(), "".to_string()]);
    }

    /// Within a single topic block, the weakest card (lowest retrievability)
    /// is served first.
    #[test]
    fn within_block_orders_weakest_card_first() {
        let mut col = Collection::new();
        let weak = add_review_card(&mut col, Some(KINETICS), 1.0, 60);
        let strong = add_review_card(&mut col, Some(KINETICS), 100_000.0, 1);

        let queued = col.get_topic_grouped_queue(DeckId(1), 0).unwrap();
        let order: Vec<CardId> = queued.cards.iter().map(|qc| qc.card.id).collect();
        assert_eq!(
            order,
            vec![weak, strong],
            "the shakier card should surface first within the block"
        );
    }

    /// fetch_limit caps the returned cards while review_count still reports the
    /// full block total; the additive path leaves the default queue intact.
    #[test]
    fn fetch_limit_caps_cards_without_touching_counts() {
        let mut col = Collection::new();
        add_review_card(&mut col, Some(KINETICS), 1.0, 60);
        add_review_card(&mut col, Some(STRUCTURE), 1.0, 60);
        add_review_card(&mut col, Some(PKA), 1.0, 60);

        let limited = col.get_topic_grouped_queue(DeckId(1), 2).unwrap();
        assert_eq!(limited.cards.len(), 2, "fetch_limit caps returned cards");
        assert_eq!(
            limited.review_count, 3,
            "count reflects the full block total"
        );
        assert_eq!(limited.new_count, 0);
        assert_eq!(limited.learning_count, 0);

        // The default queue is unchanged and still serves all three as reviews.
        let default = col.get_queued_cards(10, false).unwrap();
        assert_eq!(default.review_count, 3);
    }

    /// recent_accuracy from the revlog sharpens weakness: two topics identical
    /// in retrievability and weight rank by their graded miss rate.
    #[test]
    fn recent_accuracy_breaks_ties_between_equal_topics() {
        let mut col = Collection::new();
        let mut of: HashMap<CardId, &str> = HashMap::new();
        // Same weight (use two cards of each, identical FSRS state) so only the
        // revlog accuracy differs.
        let missed = add_review_card(&mut col, Some(PKA), 50.0, 10);
        let passed = add_review_card(&mut col, Some(METABOLISM), 50.0, 10);
        of.insert(missed, PKA);
        of.insert(passed, METABOLISM);

        // Give PKA a poor record (1 of 3) and METABOLISM a perfect one.
        // Buttons: 1 = Again (miss), 2/3/4 = Hard/Good/Easy (pass).
        log_reviews(&mut col, missed, &[1, 1, 3]);
        log_reviews(&mut col, passed, &[3, 3, 3]);

        let topics = queue_topics(&mut col, &of);
        // PKA weight 0.12 > METABOLISM 0.08, and PKA is also less accurate, so
        // it must lead regardless.
        assert_eq!(topics, vec![PKA.to_string(), METABOLISM.to_string()]);
    }

    /// Answers `cid` with Good through the standard `answer_card` path using
    /// the provided scheduling states, and returns the resulting interval.
    /// The card is answered out-of-queue (`from_queue: false`) because the
    /// topic-grouped queue is an owned, read-only ordering, not the live
    /// study queue — exactly how the reviewer grades a card it surfaced.
    fn answer_good(col: &mut Collection, cid: CardId, states: &SchedulingStates) -> u32 {
        let mut answer = CardAnswer {
            card_id: cid,
            current_state: states.current,
            new_state: states.good,
            rating: Rating::Good,
            answered_at: TimestampMillis::now(),
            milliseconds_taken: 0,
            custom_data: None,
            from_queue: false,
        };
        col.answer_card(&mut answer).unwrap();
        col.storage.get_card(cid).unwrap().unwrap().interval
    }

    /// Safety proof for spec-engine-topic-queue §8–§9 (AC5 undo, AC6 interval
    /// equivalence): the topic queue only reorders presentation, so grading a
    /// card it surfaced goes through the unchanged `answer_card` path, undoes
    /// cleanly, and produces exactly the interval the default queue would.
    #[test]
    fn answering_topic_queue_card_is_undoable_and_matches_default_interval() {
        let mut col = Collection::new();
        // Two due review cards in different topics so the topic ordering is real;
        // we grade the weaker one the queue surfaces first.
        let weak = add_review_card(&mut col, Some(KINETICS), 5.0, 30);
        add_review_card(&mut col, Some(STRUCTURE), 60.0, 3);

        let before = col.storage.get_card(weak).unwrap().unwrap();
        assert_eq!(before.ctype, CardType::Review, "precondition: review card");

        // The states the topic queue reports for the card are exactly the
        // default scheduling states — the topic path alters no scheduling.
        let topic_states = {
            let queued = col.get_topic_grouped_queue(DeckId(1), 0).unwrap();
            queued
                .cards
                .iter()
                .find(|qc| qc.card.id == weak)
                .expect("weak card is in the topic queue")
                .states
                .clone()
        };
        // SchedulingStates isn't PartialEq, so compare the per-rating CardStates
        // (which are): every next-state the topic queue offers matches the
        // default path's, so no interval differs.
        let default_states = col.get_scheduling_states(weak).unwrap();
        assert_eq!(topic_states.current, default_states.current);
        assert_eq!(topic_states.again, default_states.again);
        assert_eq!(topic_states.hard, default_states.hard);
        assert_eq!(topic_states.good, default_states.good);
        assert_eq!(topic_states.easy, default_states.easy);

        // Answer the card Good after pulling it from the topic queue.
        let topic_interval = answer_good(&mut col, weak, &topic_states);
        assert_eq!(
            col.storage
                .get_all_revlog_entries(TimestampSecs(0))
                .unwrap()
                .len(),
            1,
            "answering writes one revlog entry"
        );

        // Undo restores the card to its pristine pre-answer state and removes
        // the revlog entry — identical to undoing a default-queue answer.
        col.undo().unwrap();
        let restored = col.storage.get_card(weak).unwrap().unwrap();
        assert_eq!(restored.interval, before.interval, "interval restored");
        assert_eq!(restored.due, before.due, "due restored");
        assert_eq!(restored.reps, before.reps, "reps restored");
        assert_eq!(restored.lapses, before.lapses, "lapses restored");
        assert_eq!(restored.ctype, before.ctype, "type restored");
        assert_eq!(restored.queue, before.queue, "queue restored");
        assert_eq!(
            restored.ease_factor(),
            before.ease_factor(),
            "ease restored"
        );
        assert_eq!(
            col.storage
                .get_all_revlog_entries(TimestampSecs(0))
                .unwrap()
                .len(),
            0,
            "the revlog entry is removed on undo"
        );

        // Grade the very same card (now pristine again) Good, this time pulling
        // it from the default queue. The card's fuzz seed depends only on id and
        // reps, both restored by undo, so the scheduled interval must be
        // identical — the topic queue produced no different scheduling (AC6).
        let default_states = {
            let queued = col.get_queued_cards(10, false).unwrap();
            queued
                .cards
                .iter()
                .find(|qc| qc.card.id == weak)
                .expect("weak card is in the default queue")
                .states
                .clone()
        };
        let default_interval = answer_good(&mut col, weak, &default_states);
        assert_eq!(
            topic_interval, default_interval,
            "interval must match whether the card came from the topic or default queue"
        );
    }

    /// Writes review-kind revlog entries with the given answer buttons.
    fn log_reviews(col: &mut Collection, cid: CardId, buttons: &[u8]) {
        use crate::revlog::RevlogEntry;
        use crate::revlog::RevlogId;
        use crate::revlog::RevlogReviewKind;
        for (i, button) in buttons.iter().enumerate() {
            let entry = RevlogEntry {
                id: RevlogId(TimestampSecs::now().0 * 1000 + i as i64),
                cid,
                usn: crate::types::Usn(-1),
                button_chosen: *button,
                interval: 1,
                last_interval: 1,
                ease_factor: 2500,
                taken_millis: 1000,
                review_kind: RevlogReviewKind::Review,
            };
            // uniquify so colliding ids across cards are still inserted.
            col.storage.add_revlog_entry(&entry, true).unwrap();
        }
    }
}
