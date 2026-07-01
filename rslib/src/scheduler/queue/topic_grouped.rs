// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Topic-grouped ("Learn") review queue — Speedrun Phase 2a, decisions D3,
//! D16, D17 (see docs/plan/decisions.md).
//!
//! An *additive* alternative ordering of a deck's actionable cards: instead of
//! the default interleaved queue, the new, due-review and interday-learning
//! cards are regrouped into topic-contiguous blocks, the blocks ordered by
//! `topic_weakness × exam_weight` and the cards within a block by weakness
//! (lowest FSRS retrievability first). Surfacing new cards is what lets "Learn"
//! serve blocked first-exposure study grouped by topic (B019); intraday
//! learning steps keep their short-term timing in the default queue and are
//! skipped here.
//!
//! ## State-aware selection (mastery progression, spec-mastery-progression §6)
//!
//! On top of the ordering, the queue honors each topic's mastery state
//! ([`crate::speedrun::progression`]):
//!
//! - **Suppression:** a `SpeedrunApplication` card is dropped while its topic is
//!   below `hierarchy` (no applying before the concept is in hand).
//! - **Blocked vs mixed:** if any in-scope topic is still `learning`, the single
//!   highest-priority learning block is served (blocked first-exposure) together
//!   with already-graduated topics' due reviews, so blocking a new topic never
//!   starves earlier topics' retention (B026); otherwise the whole mixed pool is
//!   served, ordered by block priority as before.
//!
//! Both are pure presentation: no card is mutated and the state map is only
//! read, so FSRS scheduling and undo stay untouched.
//!
//! It is read-only queue construction. Gathering, limits, the [`QueuedCards`]
//! return shape and the downstream `answer_card` path are all reused verbatim,
//! so this code path never produces or alters an FSRS interval and never
//! touches the undo queue (spec §8). The only thing it decides is the order in
//! which cards are presented; each card keeps its real [`QueueEntryKind`], so a
//! new card still grades through the new-card path.
//!
//! ## Weakness proxy (Wednesday)
//!
//! Weakness is the Phase-1 [`topic_weakness`] blend of miss rate and
//! forgetting, fed two per-topic signals derived here:
//!
//! - `mean_retrievability`: the mean FSRS current retrievability over the
//!   topic's cards (same computation as the stats graphs). A card with no FSRS
//!   memory state (a new card, an SM-2 card, or one moved with "set due date")
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
use super::MainQueueEntryKind;
use super::QueueEntry;
use super::QueueEntryKind;
use super::QueuedCard;
use super::QueuedCards;
use crate::card::Card;
use crate::prelude::*;
use crate::speedrun::card_signals::card_retrievability;
use crate::speedrun::card_signals::card_topic;
use crate::speedrun::card_signals::leaf_topic_weights;
use crate::speedrun::progression::application_suppressed;
use crate::speedrun::progression::NoteKind;
use crate::speedrun::progression::TopicState;
use crate::speedrun::taxonomy::topic_weakness;

/// How many of a topic's most recent graded reviews feed `recent_accuracy`.
const RECENT_REVIEW_WINDOW: usize = 50;

/// A gathered card paired with the values the ordering and the returned
/// [`QueuedCard`] need: its topic (if any), its current FSRS retrievability and
/// its real queue kind (New / Review / interday Learning).
struct QueueCardData {
    card: Card,
    /// The taxonomy leaf id this card maps to, or `None` (unmapped).
    topic: Option<String>,
    /// FSRS current retrievability in `[0, 1]`; weakness is `1 - this`.
    retrievability: f32,
    /// The card's queue kind, threaded through so the returned [`QueuedCard`]
    /// reports New vs Review vs Learning correctly instead of a hardcoded kind.
    kind: QueueEntryKind,
    /// The card's topic's mastery state, used for blocked-vs-mixed selection
    /// (an unmapped card defaults to `learning`).
    state: TopicState,
}

impl Collection {
    /// Build the topic-grouped ("Learn") queue for `deck_id` (spec §4–§5).
    ///
    /// Reuses the standard builder for gathering + deck/limit handling, keeps
    /// the new, due-review and interday-learning cards (intraday learning steps
    /// stay in the default queue), then regroups them into topic blocks. Each
    /// card keeps its real [`QueueEntryKind`] so the reviewer grades it through
    /// the right path. `fetch_limit` caps the returned cards (`0` = no limit);
    /// the reported counts are always the full per-kind totals, mirroring
    /// [`Collection::get_queued_cards`].
    pub(crate) fn get_topic_grouped_queue(
        &mut self,
        deck_id: DeckId,
        fetch_limit: usize,
    ) -> Result<QueuedCards> {
        // build_queues() returns an *owned* queue and does not replace the live
        // study queue, so the default Practice path is left untouched. Keep each
        // entry's kind so new cards come back as New, not a hardcoded Review.
        let gathered: Vec<(CardId, QueueEntryKind)> = {
            let queues = self.build_queues(deck_id)?;
            queues
                .iter()
                .filter_map(|entry| topic_queue_kind(&entry).map(|kind| (entry.card_id(), kind)))
                .collect()
        };

        let timing = self.timing_today()?;
        let fsrs = FSRS::new(None)?;
        let leaf_weights = leaf_topic_weights();
        // One config read; topics absent from the map resolve to `learning`.
        let topic_states = self.speedrun_topic_states();

        // Pass 1: resolve each card's topic, retrievability, note kind and the
        // topic's mastery state. New cards carry no FSRS memory state, so
        // card_retrievability() returns the shared 0.9 no-memory prior for them.
        // Application cards whose topic is below `hierarchy` are suppressed here
        // (spec §6) and never enter the queue.
        let mut data: Vec<QueueCardData> = Vec::with_capacity(gathered.len());
        for (cid, kind) in gathered {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            // A card's topic is the note tag equal to a taxonomy leaf id; if
            // several match, the smallest id keeps the choice deterministic.
            let topic = card_topic(&note.tags, &leaf_weights);
            let state = topic
                .as_deref()
                .and_then(|t| topic_states.get(t).copied())
                .unwrap_or_default();
            if self.note_kind(&note)? == NoteKind::Application && application_suppressed(state) {
                continue;
            }
            let retrievability = card_retrievability(&card, &timing, &fsrs);
            data.push(QueueCardData {
                card,
                topic,
                retrievability,
                kind,
                state,
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

        // Order into priority blocks, then apply the blocked-vs-mixed selection.
        let ordered = order_cards(data, &leaf_weights, &topic_accuracy);
        let ordered = select_blocked_or_mixed(ordered);

        // Counts report what is actually servable, i.e. after suppression and
        // the blocked/mixed selection but before fetch_limit (which only caps
        // the returned cards, mirroring get_queued_cards).
        let new_count = ordered
            .iter()
            .filter(|d| d.kind == QueueEntryKind::New)
            .count();
        let learning_count = ordered
            .iter()
            .filter(|d| d.kind == QueueEntryKind::Learning)
            .count();
        let review_count = ordered
            .iter()
            .filter(|d| d.kind == QueueEntryKind::Review)
            .count();

        // Assemble the standard QueuedCards payload (states + context), exactly
        // as the default queue does, so the reviewer/answer path is reused.
        let limit = if fetch_limit == 0 {
            ordered.len()
        } else {
            fetch_limit.min(ordered.len())
        };
        let mut cards = Vec::with_capacity(limit);
        for data in ordered.into_iter().take(limit) {
            let states = self.get_scheduling_states(data.card.id)?;
            let context = new_scheduling_context(self, &data.card)?;
            cards.push(QueuedCard {
                card: data.card,
                kind: data.kind,
                states,
                context,
            });
        }

        Ok(QueuedCards {
            cards,
            new_count,
            learning_count,
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

/// The kind a queued entry contributes to the topic-grouped queue, or `None`
/// for entries the Learn ordering deliberately leaves to the default queue.
///
/// New, due Review and *interday* (day-granularity) Learning cards are the daily
/// study set we regroup by topic. Intraday learning steps (due in minutes) carry
/// their own short-term sequence and are skipped, so a card mid-step isn't pulled
/// out of order — it still surfaces normally in the default queue.
fn topic_queue_kind(entry: &QueueEntry) -> Option<QueueEntryKind> {
    match entry {
        QueueEntry::Main(e) => Some(match e.kind {
            MainQueueEntryKind::New => QueueEntryKind::New,
            MainQueueEntryKind::Review => QueueEntryKind::Review,
            MainQueueEntryKind::InterdayLearning => QueueEntryKind::Learning,
        }),
        QueueEntry::IntradayLearning(_) => None,
    }
}

/// Mean retrievability over a non-empty block; `0.0` for an empty slice.
fn mean_retrievability(cards: &[QueueCardData]) -> f32 {
    if cards.is_empty() {
        return 0.0;
    }
    cards.iter().map(|c| c.retrievability).sum::<f32>() / cards.len() as f32
}

/// Order cards into topic-contiguous blocks by `block_priority` (descending),
/// cards within a block by weakness (lowest retrievability first), with the
/// unmapped block trailing. Deterministic: ties break on topic id then card id.
/// Each card keeps its [`QueueCardData`] (so its kind survives the regrouping).
fn order_cards(
    cards: Vec<QueueCardData>,
    leaf_weights: &HashMap<String, f32>,
    topic_accuracy: &HashMap<String, Option<f32>>,
) -> Vec<QueueCardData> {
    let mut blocks: HashMap<String, Vec<QueueCardData>> = HashMap::new();
    let mut unmapped: Vec<QueueCardData> = Vec::new();
    for card in cards {
        match &card.topic {
            Some(topic) => blocks.entry(topic.clone()).or_default().push(card),
            None => unmapped.push(card),
        }
    }

    // block_priority = topic_weakness(recent_accuracy, mean_retrievability)
    //                  × exam_weight   (spec §5)
    let mut block_list: Vec<(String, f32, Vec<QueueCardData>)> = blocks
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

    let mut ordered: Vec<QueueCardData> = Vec::new();
    for (_topic, _priority, mut cards) in block_list {
        sort_block_by_weakness(&mut cards);
        ordered.extend(cards);
    }
    sort_block_by_weakness(&mut unmapped);
    ordered.extend(unmapped);
    ordered
}

/// Apply the state-aware selection (spec-mastery-progression §6): if any mapped
/// in-scope topic in the servable set is still `learning`, serve the single
/// highest-priority learning block (blocked first-exposure) **plus** the due
/// maintenance of already-graduated topics; otherwise serve the whole mixed pool
/// unchanged. `ordered` is already block-priority ordered, so the first mapped
/// `learning` card marks the highest-priority learning topic, and the retained
/// cards keep that order.
///
/// The one blocked block is the only first-exposure study served: other learning
/// topics' cards stay withheld so the learner takes on one new topic at a time.
/// But a graduated topic's due reviews and interday-learning cards keep flowing
/// (B026), because with the single Study button there is no separate Practice
/// pass to catch them, so blocking a new topic must never starve retention of
/// earlier ones. Unmapped cards never trigger the blocked phase and are excluded
/// from it (a learning block is a single mapped topic).
fn select_blocked_or_mixed(ordered: Vec<QueueCardData>) -> Vec<QueueCardData> {
    let blocked_topic = ordered.iter().find_map(|d| match &d.topic {
        Some(topic) if d.state == TopicState::Learning => Some(topic.clone()),
        _ => None,
    });
    match blocked_topic {
        Some(topic) => ordered
            .into_iter()
            .filter(|d| {
                d.topic.as_deref() == Some(topic.as_str())
                    || (d.state != TopicState::Learning
                        && matches!(
                            d.kind,
                            QueueEntryKind::Review | QueueEntryKind::Learning
                        ))
            })
            .collect(),
        None => ordered,
    }
}

/// Sort a block so the weakest cards (lowest retrievability) come first, ties
/// broken by ascending card id.
fn sort_block_by_weakness(cards: &mut [QueueCardData]) {
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

    /// Adds a brand-new card (optionally tagged `tag`) and returns its id. A
    /// freshly added note's card is already New with no memory state, so no
    /// scheduling mutation is needed — this is exactly the first-exposure card
    /// Learn must surface (B019).
    fn add_new_card(col: &mut Collection, tag: Option<&str>) -> CardId {
        let nt = col.basic_notetype();
        let mut note = nt.new_note();
        note.set_field(0, "front").unwrap();
        if let Some(tag) = tag {
            note.tags.push(tag.to_string());
        }
        col.add_note(&mut note, DeckId(1)).unwrap();
        col.storage.card_ids_of_notes(&[note.id]).unwrap()[0]
    }

    /// Graduate `topics` out of the blocked `learning` phase into the mixed
    /// pool by setting each to `state`. A fresh topic defaults to `learning`,
    /// which would serve only a single blocked block; these mixed-pool ordering
    /// tests need the topics graduated so every block is served.
    fn graduate(col: &mut Collection, topics: &[&str], state: TopicState) {
        for topic in topics {
            col.set_speedrun_topic_state(topic, state).unwrap();
        }
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

    /// The ids of the cards the queue serves, in order.
    fn queue_ids(col: &mut Collection) -> Vec<CardId> {
        col.get_topic_grouped_queue(DeckId(1), 0)
            .unwrap()
            .cards
            .iter()
            .map(|qc| qc.card.id)
            .collect()
    }

    /// Create and persist a minimal note type with the given `name`, so the
    /// queue's note-kind logic classifies its cards (e.g. "SpeedrunApplication").
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

    /// Add a due review card of `nt`, tagged `tag`, with an FSRS memory state.
    fn add_review_card_of(
        col: &mut Collection,
        nt: &Notetype,
        tag: &str,
        stability: f32,
        elapsed_days: i64,
    ) -> CardId {
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
        cid
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
        graduate(
            &mut col,
            &[KINETICS, STRUCTURE, PKA],
            TopicState::Practicing,
        );

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
        graduate(
            &mut col,
            &[PKA, METABOLISM, KINETICS],
            TopicState::Practicing,
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
        graduate(&mut col, &[KINETICS], TopicState::Practicing);

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

    /// B019: new cards (no FSRS memory state) are surfaced by Learn, grouped
    /// into their topic block alongside due reviews, and returned with their
    /// real kind (New) rather than a hardcoded Review.
    #[test]
    fn new_cards_are_included_grouped_and_tagged_new() {
        let mut col = Collection::new();
        // KINETICS (weight 0.18) holds a weak due review and a brand-new card;
        // STRUCTURE (0.15) holds a strongly-retained review, so KINETICS leads.
        let kinetics_review = add_review_card(&mut col, Some(KINETICS), 1.0, 60);
        let kinetics_new = add_new_card(&mut col, Some(KINETICS));
        let structure_review = add_review_card(&mut col, Some(STRUCTURE), 100_000.0, 1);
        graduate(&mut col, &[KINETICS, STRUCTURE], TopicState::Practicing);

        let queued = col.get_topic_grouped_queue(DeckId(1), 0).unwrap();

        // Counts report the per-kind totals, not everything lumped as review.
        assert_eq!(queued.new_count, 1, "the new card is counted as new");
        assert_eq!(queued.review_count, 2);
        assert_eq!(queued.learning_count, 0);

        let order: Vec<CardId> = queued.cards.iter().map(|qc| qc.card.id).collect();
        assert_eq!(
            order,
            vec![kinetics_review, kinetics_new, structure_review],
            "the new card joins its KINETICS block (weakest review first, then \
             the 0.9-prior new card); the strong STRUCTURE block trails"
        );

        let kind_of = |id: CardId| {
            queued
                .cards
                .iter()
                .find(|qc| qc.card.id == id)
                .unwrap()
                .kind
        };
        assert_eq!(
            kind_of(kinetics_new),
            QueueEntryKind::New,
            "kind threaded through as New"
        );
        assert_eq!(kind_of(kinetics_review), QueueEntryKind::Review);
        assert_eq!(kind_of(structure_review), QueueEntryKind::Review);
    }

    /// fetch_limit caps the returned cards while review_count still reports the
    /// full block total; the additive path leaves the default queue intact.
    #[test]
    fn fetch_limit_caps_cards_without_touching_counts() {
        let mut col = Collection::new();
        add_review_card(&mut col, Some(KINETICS), 1.0, 60);
        add_review_card(&mut col, Some(STRUCTURE), 1.0, 60);
        add_review_card(&mut col, Some(PKA), 1.0, 60);
        graduate(
            &mut col,
            &[KINETICS, STRUCTURE, PKA],
            TopicState::Practicing,
        );

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
        graduate(&mut col, &[PKA, METABOLISM], TopicState::Practicing);

        let topics = queue_topics(&mut col, &of);
        // PKA weight 0.12 > METABOLISM 0.08, and PKA is also less accurate, so
        // it must lead regardless.
        assert_eq!(topics, vec![PKA.to_string(), METABOLISM.to_string()]);
    }

    /// (c) An application card is suppressed while its topic is below
    /// `hierarchy` (learning/practicing) and served once it reaches `hierarchy`;
    /// a non-application card in the same topic is never suppressed.
    #[test]
    fn application_cards_suppressed_below_hierarchy() {
        let mut col = Collection::new();
        let app_nt = add_notetype_named(&mut col, "SpeedrunApplication");
        let app = add_review_card_of(&mut col, &app_nt, KINETICS, 10.0, 5);
        let basic = add_review_card(&mut col, Some(KINETICS), 10.0, 5);

        // practicing is still below hierarchy: the application card is dropped,
        // the basic card stays. (practicing, not learning, so it's mixed — this
        // isolates suppression from the blocked-phase filter.)
        graduate(&mut col, &[KINETICS], TopicState::Practicing);
        let ids = queue_ids(&mut col);
        assert!(ids.contains(&basic), "non-application card always served");
        assert!(
            !ids.contains(&app),
            "application card suppressed below hierarchy"
        );

        // At hierarchy the application card joins the queue.
        graduate(&mut col, &[KINETICS], TopicState::Hierarchy);
        let ids = queue_ids(&mut col);
        assert!(ids.contains(&app), "application card served at hierarchy");
        assert!(ids.contains(&basic));

        // Still served at mastering (scaffold removed, but card present).
        graduate(&mut col, &[KINETICS], TopicState::Mastering);
        assert!(queue_ids(&mut col).contains(&app));
    }

    /// (d) Blocked vs mixed selection: while any topic is `learning` only the
    /// single highest-priority learning block is served; once every topic has
    /// graduated the whole mixed pool is served in priority order.
    #[test]
    fn blocked_phase_serves_one_learning_block_then_mixes() {
        let mut col = Collection::new();
        let mut of: HashMap<CardId, &str> = HashMap::new();
        of.insert(add_review_card(&mut col, Some(KINETICS), 10.0, 5), KINETICS);
        of.insert(
            add_review_card(&mut col, Some(STRUCTURE), 10.0, 5),
            STRUCTURE,
        );
        of.insert(add_review_card(&mut col, Some(PKA), 10.0, 5), PKA);

        // All fresh (learning) → blocked on the heaviest topic (KINETICS 0.18).
        assert_eq!(
            queue_topics(&mut col, &of),
            vec![KINETICS.to_string()],
            "blocked: only the highest-priority learning block is served"
        );

        // Graduating KINETICS doesn't open the pool — STRUCTURE/PKA are still
        // learning, so the queue blocks on the next-heaviest (STRUCTURE 0.15).
        // But KINETICS is now graduated, so its due review keeps flowing as
        // maintenance (B026) ahead of the STRUCTURE block by priority (0.18).
        graduate(&mut col, &[KINETICS], TopicState::Practicing);
        assert_eq!(
            queue_topics(&mut col, &of),
            vec![KINETICS.to_string(), STRUCTURE.to_string()],
            "blocked on STRUCTURE, but graduated KINETICS' due review isn't starved"
        );

        // Once nothing is learning, the mixed pool serves every block by
        // priority (KINETICS 0.18 > STRUCTURE 0.15 > PKA 0.12).
        graduate(&mut col, &[STRUCTURE, PKA], TopicState::Practicing);
        let topics = queue_topics(&mut col, &of);
        let runs: Vec<String> = topics
            .iter()
            .enumerate()
            .filter(|(i, t)| *i == 0 || &topics[i - 1] != *t)
            .map(|(_, t)| t.clone())
            .collect();
        assert_eq!(
            runs,
            vec![KINETICS.to_string(), STRUCTURE.to_string(), PKA.to_string()],
            "mixed: all graduated blocks served in priority order"
        );
    }

    /// B026: while a new topic is blocked, a *graduated* topic's due review keeps
    /// flowing (retention isn't starved), but another *learning* topic's new
    /// first-exposure card stays withheld (one new topic at a time).
    #[test]
    fn blocked_phase_serves_graduated_reviews_but_withholds_other_new_topics() {
        let mut col = Collection::new();
        // KINETICS: graduated, with a weak due review that must keep flowing.
        let kin_review = add_review_card(&mut col, Some(KINETICS), 1.0, 60);
        graduate(&mut col, &[KINETICS], TopicState::Hierarchy);
        // STRUCTURE + PKA are both still learning. STRUCTURE (0.15) outranks PKA
        // (0.12), so STRUCTURE is the single blocked first-exposure block.
        let structure_new = add_new_card(&mut col, Some(STRUCTURE));
        let pka_new = add_new_card(&mut col, Some(PKA));

        let ids = queue_ids(&mut col);
        assert!(
            ids.contains(&kin_review),
            "a graduated topic's due review is not starved by the blocked phase"
        );
        assert!(
            ids.contains(&structure_new),
            "the single highest-priority learning block is served"
        );
        assert!(
            !ids.contains(&pka_new),
            "another learning topic's new card stays withheld"
        );
    }

    /// Button-merge fallback (spec-mastery-progression §6): a deck with no
    /// Speedrun topics leaves every card unmapped, so the blocked phase never
    /// triggers and nothing is suppressed — the topic-grouped queue serves
    /// exactly the same actionable cards as the default queue. This is what lets
    /// the single "Study" button behave like normal study on a normal deck.
    #[test]
    fn non_speedrun_deck_matches_default_queue() {
        let mut col = Collection::new();
        let a = add_review_card(&mut col, None, 5.0, 30);
        let b = add_review_card(&mut col, Some("not::a::taxonomy::tag"), 50.0, 3);
        let c = add_new_card(&mut col, None);

        let mut topic_ids = queue_ids(&mut col);
        topic_ids.sort();

        let mut default_ids: Vec<CardId> = col
            .get_queued_cards(100, false)
            .unwrap()
            .cards
            .iter()
            .map(|qc| qc.card.id)
            .collect();
        default_ids.sort();

        let mut expected = vec![a, b, c];
        expected.sort();
        assert_eq!(
            topic_ids, expected,
            "all actionable cards are served (no blocked phase, no suppression)"
        );
        assert_eq!(
            topic_ids, default_ids,
            "no Speedrun topics: Study serves the same set as the default queue"
        );
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
        graduate(&mut col, &[KINETICS, STRUCTURE], TopicState::Practicing);

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

    /// Answers `cid` Good at a fixed instant via the standard path (out of
    /// queue, as the reviewer does for a card the topic queue surfaced) and
    /// returns the resulting card. A fixed `at` keeps a learning card's due time
    /// comparable across two separate answers.
    fn answer_good_at(
        col: &mut Collection,
        cid: CardId,
        states: &SchedulingStates,
        at: TimestampMillis,
    ) -> Card {
        let mut answer = CardAnswer {
            card_id: cid,
            current_state: states.current,
            new_state: states.good,
            rating: Rating::Good,
            answered_at: at,
            milliseconds_taken: 0,
            custom_data: None,
            from_queue: false,
        };
        col.answer_card(&mut answer).unwrap();
        col.storage.get_card(cid).unwrap().unwrap()
    }

    /// The scheduling-relevant fields of a card, for comparing two answer paths.
    fn sched_outcome(card: &Card) -> (CardType, CardQueue, i32, u32, u32, u32) {
        (
            card.ctype,
            card.queue,
            card.due,
            card.interval,
            card.reps,
            card.remaining_steps,
        )
    }

    /// B019 safety proof for new cards: a new card the topic queue surfaces
    /// reports the same scheduling states as the default new-card path, grades
    /// through the unchanged `answer_card` path to an identical outcome, and the
    /// answer undoes cleanly back to the pristine new card.
    #[test]
    fn answering_new_card_from_topic_queue_matches_default_and_is_undoable() {
        let mut col = Collection::new();
        // A new card to grade, plus a due review in another topic so the queue
        // is a real topic-grouped ordering rather than a single card.
        let new_card = add_new_card(&mut col, Some(KINETICS));
        add_review_card(&mut col, Some(STRUCTURE), 60.0, 3);
        graduate(&mut col, &[KINETICS, STRUCTURE], TopicState::Practicing);

        let before = col.storage.get_card(new_card).unwrap().unwrap();
        assert_eq!(before.ctype, CardType::New, "precondition: new card");

        // The topic queue surfaces it as New with the *same* scheduling states
        // the default new-card path computes — the topic path schedules nothing.
        let topic_states = {
            let queued = col.get_topic_grouped_queue(DeckId(1), 0).unwrap();
            let qc = queued
                .cards
                .iter()
                .find(|qc| qc.card.id == new_card)
                .expect("the new card is in the topic queue");
            assert_eq!(qc.kind, QueueEntryKind::New, "carried through as New");
            qc.states.clone()
        };
        let default_states = col.get_scheduling_states(new_card).unwrap();
        assert_eq!(topic_states.current, default_states.current);
        assert_eq!(topic_states.again, default_states.again);
        assert_eq!(topic_states.hard, default_states.hard);
        assert_eq!(topic_states.good, default_states.good);
        assert_eq!(topic_states.easy, default_states.easy);

        // Grade Good at a fixed instant from the topic queue; capture the result.
        let at = TimestampMillis::now();
        let topic_outcome = sched_outcome(&answer_good_at(&mut col, new_card, &topic_states, at));
        assert_eq!(
            col.storage
                .get_all_revlog_entries(TimestampSecs(0))
                .unwrap()
                .len(),
            1,
            "answering a new card writes one revlog entry"
        );

        // Undo restores the pristine new card and removes the revlog entry.
        col.undo().unwrap();
        let restored = col.storage.get_card(new_card).unwrap().unwrap();
        assert_eq!(restored.ctype, before.ctype, "type restored to New");
        assert_eq!(restored.queue, before.queue, "queue restored to New");
        assert_eq!(restored.due, before.due, "due restored");
        assert_eq!(restored.reps, before.reps, "reps restored");
        assert_eq!(restored.remaining_steps, before.remaining_steps);
        assert_eq!(
            col.storage
                .get_all_revlog_entries(TimestampSecs(0))
                .unwrap()
                .len(),
            0,
            "the revlog entry is removed on undo"
        );

        // Grading the same pristine card from the default queue at the same
        // instant produces an identical outcome — the topic queue scheduled the
        // new card exactly as the default new-card path would (AC6).
        let default_states = {
            let queued = col.get_queued_cards(10, false).unwrap();
            queued
                .cards
                .iter()
                .find(|qc| qc.card.id == new_card)
                .expect("the new card is in the default queue")
                .states
                .clone()
        };
        let default_outcome =
            sched_outcome(&answer_good_at(&mut col, new_card, &default_states, at));
        assert_eq!(
            topic_outcome, default_outcome,
            "a new card is scheduled identically via the topic or default queue"
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
