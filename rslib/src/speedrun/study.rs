// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Bespoke Speedrun study backend, shared by desktop and mobile.
//!
//! This is the engine behind the animated `speedrun-review` study screen. It
//! was ported from desktop-only Python
//! (`pylib/anki/speedrun/{study,materialize}.py`) into the shared Rust layer so
//! both the Qt desktop app and the AnkiDroid fork drive the same RPCs, over the
//! same backend, with no per-platform study code.
//!
//! Two concerns are kept deliberately separate:
//!
//! - **FSRS owns *timing*** (when a concept's card is due). Each authored
//!   concept is *materialized* into one real Anki card of a minimal
//!   [`ITEM_NOTETYPE_NAME`] note type, keyed by its `ConceptId`, so the
//!   standard scheduler serves it and [`Collection::grade_now`] grades it with
//!   genuine FSRS intervals.
//! - **This module owns *mode*** (what interaction renders): a per-concept
//!   mastery record stored **on each concept's `SpeedrunItem` card**, in the
//!   card's `custom_data` JSON (keys `sr*`). It never produces an FSRS
//!   interval.
//!
//! Mastery used to live in one monolithic collection-config blob
//! ([`STUDY_PROGRESS_CONFIG_KEY`]). Config syncs whole-table, newest-collection
//! -wins, so two devices editing different concepts offline would clobber each
//! other on sync. Card `custom_data` instead merges per card (newest card
//! wins, [`crate::sync::collection::chunks`]), so concurrent edits to different
//! concepts each survive. A one-time migration
//! ([`Collection::speedrun_migrate_progress_if_needed`]) folds any legacy blob
//! onto the cards on first read.
//!
//! Each concept walks a four-state mastery ladder ([`TopicState`]); the
//! internal `hierarchy` state displays as "Applying" in the UI:
//!
//! ```text
//! learning -> practicing -> hierarchy(Applying) -> mastering
//! ```
//!
//! - **Learning is topic-gated**: a concept stays `learning` until every
//!   concept in its authored leaf node has been seen, at which point they all
//!   flip to `practicing` together (`record_learned`).
//! - **Later stages are per-concept**: an answer advances one state once the
//!   concept's recent ratings clear the mastery signal, or demotes one on
//!   `Again`. Demotion floors at `practicing`, so a lapse never re-enters the
//!   topic block (`record_answer`).
//!
//! Everything reads/writes the authoring + progress config and the materialized
//! cards; the RPC surface exchanges JSON so the screen's `lib.ts` is unchanged.

use std::collections::HashMap;
use std::collections::HashSet;

use fsrs::FSRS;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

use crate::config::BoolKey;
use crate::notetype::Notetype;
use crate::prelude::*;
use crate::scheduler::timespan::answer_button_time_collapsible;
use crate::search::SearchNode;
use crate::search::SortMode;
use crate::speedrun::card_signals::card_retrievability;

/// One concept's position on the four-state mastery ladder. Serialized
/// lowercase in the study-progress store; an absent entry defaults to
/// `Learning`. The internal `hierarchy` state displays as "Applying".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TopicState {
    #[default]
    Learning,
    Practicing,
    Hierarchy,
    Mastering,
}

impl TopicState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            TopicState::Learning => "learning",
            TopicState::Practicing => "practicing",
            TopicState::Hierarchy => "hierarchy",
            TopicState::Mastering => "mastering",
        }
    }
}

/// Legacy collection-config key that once held the per-deck, per-concept
/// mastery map (`{ "<deckId>": { "<conceptId>": { state, seen, ratings,
/// appCorrect, appTotal } } }`). Retained only so the one-time migration can
/// fold it onto the cards; nothing reads it for live study anymore.
const STUDY_PROGRESS_CONFIG_KEY: &str = "speedrun_study_progress";
/// Collection-config flag set once the legacy `speedrun_study_progress` blob
/// has been folded onto the cards' `custom_data`. Guards the migration so it
/// runs at most once per collection.
const MASTERY_MIGRATED_CONFIG_KEY: &str = "speedrun_mastery_migrated";
/// Collection-config key holding the authored hierarchy blob per deck (written
/// by the authoring editor; read here to drive materialize + learning blocks).
const AUTHORING_CONFIG_KEY: &str = "speedrun_authoring";

// Per-card `custom_data` keys for the mastery record. Each is <= 8 bytes and
// the whole serialized object stays well under the 100-byte `custom_data`
// budget (see `validate_custom_data`), which is why `ratings` is packed as a
// digit string and the window is capped (see [`MASTERY_REVIEW_WINDOW`]). Every
// key is skipped when its value is the zero/default, so an unstudied card
// carries no `custom_data` at all.
const CD_STATE: &str = "srs";
const CD_SEEN: &str = "srn";
const CD_RATINGS: &str = "srr";
const CD_APP_CORRECT: &str = "srac";
const CD_APP_TOTAL: &str = "srat";

/// The minimal note type each authored concept is mirrored into. Field order is
/// the contract: `ConceptId` (index 0) keys the card back to the concept, and
/// `Title` (index 1) is display-only. Must match the legacy desktop note type
/// so existing materialized cards keep mapping.
const ITEM_NOTETYPE_NAME: &str = "SpeedrunItem";
const ITEM_FIELD_CONCEPT_ID: usize = 0;
const ITEM_FIELD_TITLE: usize = 1;
const ITEM_QFMT: &str = "<div class=\"speedrun-item\">{{Title}}</div>";
const ITEM_AFMT: &str = "{{FrontSide}}";

/// Minimum ≥Good rate over a concept's recent ratings needed to advance one
/// state. Tunable; mirrors `progression.rs`.
const ACC_THRESHOLD: f32 = 0.8;
/// Minimum recorded answers before a concept can advance.
const MIN_REPS: usize = 2;
/// How many of the most recent ratings feed the advancement signal, and the cap
/// on how many are persisted per card. Kept small so the packed `srr` string
/// plus the other mastery keys stay inside the 100-byte `custom_data` budget
/// (see the `CD_*` keys). 32 recent ratings is ample for an "80% of recent"
/// signal; it was 50 while mastery lived in the unbounded config blob.
const MASTERY_REVIEW_WINDOW: usize = 32;

/// Difficulty rating as sent by the screen: 1..4 = Again/Hard/Good/Easy.
const RATING_AGAIN: i32 = 1;
const RATING_GOOD: i32 = 3;

/// The mastery ladder, low to high. Advancement walks up one rung; demotion
/// walks down one but never below index [`PRACTICING_INDEX`] (practicing), so a
/// lapse never drops a concept back into the learning topic block.
const LADDER: [TopicState; 4] = [
    TopicState::Learning,
    TopicState::Practicing,
    TopicState::Hierarchy,
    TopicState::Mastering,
];
const PRACTICING_INDEX: usize = 1;

fn ladder_index(state: TopicState) -> usize {
    LADDER.iter().position(|s| *s == state).unwrap_or(0)
}

fn advanced(state: TopicState) -> TopicState {
    LADDER[(ladder_index(state) + 1).min(LADDER.len() - 1)]
}

fn demoted(state: TopicState) -> TopicState {
    LADDER[ladder_index(state).saturating_sub(1).max(PRACTICING_INDEX)]
}

/// True when recent ratings clear the advancement signal: at least [`MIN_REPS`]
/// of the last [`MASTERY_REVIEW_WINDOW`] answers, with a ≥Good rate of at least
/// [`ACC_THRESHOLD`].
fn signal_cleared(ratings: &[i32]) -> bool {
    let window = if ratings.len() > MASTERY_REVIEW_WINDOW {
        &ratings[ratings.len() - MASTERY_REVIEW_WINDOW..]
    } else {
        ratings
    };
    if window.len() < MIN_REPS {
        return false;
    }
    let good = window.iter().filter(|r| **r >= RATING_GOOD).count();
    good as f32 / window.len() as f32 >= ACC_THRESHOLD
}

/// One concept's mastery state. Held in memory during a study call and
/// persisted on the concept's `SpeedrunItem` card via
/// [`ConceptEntry::write_into_custom_data`]. The `Serialize`/`Deserialize`
/// derives are used only to read the **legacy** config blob during migration
/// (shape `{ state, seen, ratings, appCorrect, appTotal }`); the live per-card
/// form uses the compact `CD_*` keys instead. All fields default, so a legacy
/// entry written before Performance persistence existed round-trips unchanged
/// (the two counts start at 0).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ConceptEntry {
    #[serde(default)]
    pub(crate) state: TopicState,
    #[serde(default)]
    pub(crate) seen: bool,
    #[serde(default)]
    ratings: Vec<i32>,
    /// Application-stage attempts answered correctly (rating ≥ Good). Only
    /// grows while the concept is at Applying/Mastering
    /// (`hierarchy`/`mastering`), the stages that render an application
    /// problem.
    #[serde(default, rename = "appCorrect")]
    pub(crate) app_correct: u32,
    /// Total application-stage attempts, correct or not; the denominator of the
    /// concept's Performance accuracy.
    #[serde(default, rename = "appTotal")]
    pub(crate) app_total: u32,
}

/// True for the empty/absent `custom_data` forms Anki uses interchangeably.
fn custom_data_is_empty(s: &str) -> bool {
    matches!(s, "" | "{}")
}

impl ConceptEntry {
    /// Read a concept's mastery record from its card's `custom_data` JSON. An
    /// absent card or absent key defaults to learning/unseen with no evidence,
    /// so an unstudied card round-trips to a default entry.
    fn from_custom_data(custom_data: &str) -> Self {
        let map: serde_json::Map<String, Value> = if custom_data_is_empty(custom_data) {
            serde_json::Map::new()
        } else {
            serde_json::from_str(custom_data).unwrap_or_default()
        };
        let state = match map.get(CD_STATE).and_then(Value::as_u64).unwrap_or(0) {
            1 => TopicState::Practicing,
            2 => TopicState::Hierarchy,
            3 => TopicState::Mastering,
            _ => TopicState::Learning,
        };
        let ratings = map
            .get(CD_RATINGS)
            .and_then(Value::as_str)
            .unwrap_or("")
            .chars()
            .filter_map(|c| c.to_digit(10).map(|d| d as i32))
            .collect();
        ConceptEntry {
            state,
            seen: map.get(CD_SEEN).and_then(Value::as_u64).unwrap_or(0) != 0,
            ratings,
            app_correct: map.get(CD_APP_CORRECT).and_then(Value::as_u64).unwrap_or(0) as u32,
            app_total: map.get(CD_APP_TOTAL).and_then(Value::as_u64).unwrap_or(0) as u32,
        }
    }

    /// Merge this record into a card's existing `custom_data`, preserving any
    /// unrelated keys, and return the new JSON string (`""` when it collapses
    /// to empty). Zero/default fields are written as *absent* so the object
    /// stays minimal, and `ratings` is packed to at most
    /// [`MASTERY_REVIEW_WINDOW`] single digits to respect the 100-byte
    /// budget.
    fn write_into_custom_data(&self, custom_data: &str) -> String {
        let mut map: serde_json::Map<String, Value> = if custom_data_is_empty(custom_data) {
            serde_json::Map::new()
        } else {
            serde_json::from_str(custom_data).unwrap_or_default()
        };
        set_or_remove(&mut map, CD_STATE, ladder_index(self.state) as u64);
        set_or_remove(&mut map, CD_SEEN, self.seen as u64);
        set_or_remove(&mut map, CD_APP_CORRECT, self.app_correct as u64);
        set_or_remove(&mut map, CD_APP_TOTAL, self.app_total as u64);
        let packed: String = self
            .ratings
            .iter()
            .rev()
            .take(MASTERY_REVIEW_WINDOW)
            .rev()
            .filter_map(|r| char::from_digit(*r as u32, 10))
            .collect();
        if packed.is_empty() {
            map.remove(CD_RATINGS);
        } else {
            map.insert(CD_RATINGS.to_string(), Value::from(packed));
        }
        if map.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&map).unwrap_or_default()
        }
    }
}

/// Insert `key=value` when non-zero, else remove it, so default fields never
/// take up room in the card's `custom_data`.
fn set_or_remove(map: &mut serde_json::Map<String, Value>, key: &str, value: u64) {
    if value == 0 {
        map.remove(key);
    } else {
        map.insert(key.to_string(), Value::from(value));
    }
}

/// `conceptId -> entry` for one deck.
pub(crate) type ConceptMap = HashMap<String, ConceptEntry>;
/// `deckId -> conceptMap`; the legacy value once stored under
/// [`STUDY_PROGRESS_CONFIG_KEY`], read only during migration.
type StudyProgressStore = HashMap<String, ConceptMap>;

// --- Authored-hierarchy view (read-only, minimal) --------------------------
//
// Only the fields the study logic needs are deserialized; unknown fields (the
// concept `content`, `problems`, etc. that the frontend renders) are ignored.

#[derive(Debug, Clone, Default, Deserialize)]
struct AuthoredConcept {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AuthoredNode {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    children: Vec<AuthoredNode>,
    #[serde(default)]
    concepts: Vec<AuthoredConcept>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AuthoredHierarchy {
    #[serde(default)]
    root: Option<AuthoredNode>,
}

fn walk_nodes<'a>(node: &'a AuthoredNode, out: &mut Vec<&'a AuthoredNode>) {
    out.push(node);
    for child in &node.children {
        walk_nodes(child, out);
    }
}

impl AuthoredHierarchy {
    fn all_nodes(&self) -> Vec<&AuthoredNode> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            walk_nodes(root, &mut out);
        }
        out
    }

    /// Every node that directly holds concepts (a topic block), in tree order.
    fn concept_leaves(&self) -> Vec<&AuthoredNode> {
        self.all_nodes()
            .into_iter()
            .filter(|n| !n.concepts.is_empty())
            .collect()
    }
}

/// The non-empty concept ids of a node, in order.
fn node_concept_ids(node: &AuthoredNode) -> Vec<String> {
    node.concepts
        .iter()
        .filter(|c| !c.id.is_empty())
        .map(|c| c.id.clone())
        .collect()
}

/// The first leaf topic still holding a `learning` concept, as the block to
/// teach, or `None` when every concept has been learned.
fn next_learning_block(hierarchy: &AuthoredHierarchy, progress: &ConceptMap) -> Option<Value> {
    for node in hierarchy.concept_leaves() {
        let ids = node_concept_ids(node);
        if ids.is_empty() {
            continue;
        }
        let any_learning = ids.iter().any(|id| {
            progress
                .get(id)
                .map(|e| e.state)
                .unwrap_or(TopicState::Learning)
                == TopicState::Learning
        });
        if any_learning {
            let learned = ids
                .iter()
                .filter(|id| progress.get(*id).map(|e| e.seen).unwrap_or(false))
                .count();
            return Some(json!({
                "kind": "learning_block",
                "topicNodeId": node.id,
                "conceptIds": ids,
                "learnedCount": learned,
                "totalCount": ids.len(),
            }));
        }
    }
    None
}

/// The outcome of a reconcile pass, for tests and logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReconcileOutcome {
    pub created: usize,
    pub removed: usize,
    pub total: usize,
}

impl Collection {
    // --- per-card mastery read/write ---------------------------------------

    /// `conceptId -> (cardId, entry)` for every materialized concept in the
    /// deck, reading each concept's mastery straight off its `SpeedrunItem`
    /// card's `custom_data`. Concepts with no card yet (deck never reconciled)
    /// are simply absent, matching the old "absent = default" contract.
    fn speedrun_card_mastery(
        &mut self,
        deck_id: DeckId,
    ) -> Result<HashMap<String, (CardId, ConceptEntry)>> {
        let Some(notetype_id) = self
            .get_notetype_by_name(ITEM_NOTETYPE_NAME)?
            .map(|nt| nt.id)
        else {
            return Ok(HashMap::new());
        };
        let mut out = HashMap::new();
        for cid in self.search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)? {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            if note.notetype_id != notetype_id {
                continue;
            }
            let concept_id = note
                .fields()
                .get(ITEM_FIELD_CONCEPT_ID)
                .cloned()
                .unwrap_or_default();
            if concept_id.is_empty() {
                continue;
            }
            out.insert(
                concept_id,
                (cid, ConceptEntry::from_custom_data(&card.custom_data)),
            );
        }
        Ok(out)
    }

    /// The deck's per-concept mastery map (`conceptId -> entry`), read from the
    /// concept cards' `custom_data`. Runs the one-time legacy-blob migration on
    /// first access so pre-existing progress is preserved.
    pub(crate) fn speedrun_deck_progress(&mut self, deck_id: DeckId) -> Result<ConceptMap> {
        self.speedrun_migrate_progress_if_needed()?;
        Ok(self
            .speedrun_card_mastery(deck_id)?
            .into_iter()
            .map(|(concept_id, (_cid, entry))| (concept_id, entry))
            .collect())
    }

    /// Persist mastery records onto their cards. Non-undoable (the mastery
    /// record is bookkeeping beside the FSRS answer, so it must not push an
    /// undo entry — [`Op::SkipUndo`] preserves any prior undoable grade),
    /// and each write bumps mtime/usn so the card (and its merged mastery)
    /// syncs. Unchanged cards are skipped, so no redundant mtime churn or
    /// sync traffic.
    fn speedrun_persist_entries(&mut self, entries: &[(CardId, ConceptEntry)]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        self.transact(Op::SkipUndo, |col| {
            let usn = col.usn()?;
            for (card_id, entry) in entries {
                let mut card = col.storage.get_card(*card_id)?.or_not_found(*card_id)?;
                let new_custom_data = entry.write_into_custom_data(&card.custom_data);
                if new_custom_data != card.custom_data {
                    card.custom_data = new_custom_data;
                    card.set_modified(usn);
                    col.storage.update_card(&card)?;
                }
            }
            Ok(())
        })?;
        Ok(())
    }

    /// One-time migration: fold any legacy `speedrun_study_progress` config
    /// blob onto the concept cards' `custom_data`, so progress recorded
    /// before the move to per-card (sync-mergeable) storage is preserved.
    /// Guarded by a config flag so it runs at most once. The legacy blob is
    /// intentionally left in place as a backup — nothing reads it for live
    /// study anymore, so a now-stale copy syncing around is harmless.
    fn speedrun_migrate_progress_if_needed(&mut self) -> Result<()> {
        if self
            .get_config_optional::<bool, _>(MASTERY_MIGRATED_CONFIG_KEY)
            .unwrap_or(false)
        {
            return Ok(());
        }
        let store: StudyProgressStore = self
            .get_config_optional(STUDY_PROGRESS_CONFIG_KEY)
            .unwrap_or_default();
        for (deck_str, concept_map) in &store {
            let Ok(raw) = deck_str.parse::<i64>() else {
                continue;
            };
            let deck_id = DeckId(raw);
            // Materialize the deck's cards so there is somewhere to write, then
            // map each legacy entry onto its concept's card.
            self.speedrun_reconcile(deck_id)?;
            let cards = self.speedrun_card_mastery(deck_id)?;
            let to_write: Vec<(CardId, ConceptEntry)> = concept_map
                .iter()
                .filter_map(|(concept_id, entry)| {
                    cards.get(concept_id).map(|(cid, _)| (*cid, entry.clone()))
                })
                .collect();
            self.speedrun_persist_entries(&to_write)?;
        }
        self.set_config_json(MASTERY_MIGRATED_CONFIG_KEY, &true, false)?;
        Ok(())
    }

    fn speedrun_authoring_store(&self) -> HashMap<String, Value> {
        self.get_config_optional(AUTHORING_CONFIG_KEY)
            .unwrap_or_default()
    }

    fn speedrun_authored_hierarchy(&self, deck_id: DeckId) -> AuthoredHierarchy {
        self.speedrun_authoring_store()
            .get(&deck_id.0.to_string())
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    // --- public study logic (JSON-shaped, matching the frontend) -----------

    /// The mastery state of every authored concept in the deck:
    /// `{ progress: { <conceptId>: { state, seen } } }`. An absent concept
    /// defaults to learning/unseen, so the screen always gets a complete map.
    pub(crate) fn speedrun_study_state(&mut self, deck_id: DeckId) -> Result<Value> {
        let progress = self.speedrun_deck_progress(deck_id)?;
        let hierarchy = self.speedrun_authored_hierarchy(deck_id);
        let mut out = serde_json::Map::new();
        for node in hierarchy.concept_leaves() {
            for id in node_concept_ids(node) {
                let entry = progress.get(&id).cloned().unwrap_or_default();
                out.insert(
                    id,
                    json!({ "state": entry.state.as_str(), "seen": entry.seen }),
                );
            }
        }
        Ok(json!({ "progress": Value::Object(out) }))
    }

    /// Reconcile the deck's cards, then return the learning block for the first
    /// topic still being taught, else the next FSRS-due card, else done.
    pub(crate) fn speedrun_next_card(&mut self, deck_id: DeckId) -> Result<Value> {
        self.speedrun_reconcile(deck_id)?;
        let hierarchy = self.speedrun_authored_hierarchy(deck_id);
        let progress = self.speedrun_deck_progress(deck_id)?;
        if let Some(block) = next_learning_block(&hierarchy, &progress) {
            return Ok(block);
        }
        // No learning block: serve the next FSRS-due card the scheduler would.
        let Some(card) = self.speedrun_peek_next_card(deck_id)? else {
            return Ok(json!({ "kind": "done" }));
        };
        let note = self
            .storage
            .get_note(card.note_id)?
            .or_not_found(card.note_id)?;
        let concept_id = self.speedrun_concept_id_for_note(&note)?;
        // A card served as a review is past learning, so an absent entry means
        // practicing (never learning).
        let state = progress
            .get(&concept_id)
            .map(|e| e.state)
            .unwrap_or(TopicState::Practicing);
        Ok(json!({
            "kind": "review",
            "cardId": card.id.0.to_string(),
            "conceptId": concept_id,
            "state": state.as_str(),
        }))
    }

    /// Mark concepts seen; when every concept in a touched leaf topic is seen,
    /// flip that whole topic `learning -> practicing` together (topic-gated).
    /// Returns `{ upgraded, from, to, conceptIds }` — the concepts flipped this
    /// call (empty until the topic is fully learned).
    pub(crate) fn speedrun_record_learned(
        &mut self,
        deck_id: DeckId,
        concept_ids: &[String],
    ) -> Result<Value> {
        // Ensure every concept has a card to carry its mastery, then migrate any
        // legacy progress, before reading the current per-card map.
        self.speedrun_reconcile(deck_id)?;
        self.speedrun_migrate_progress_if_needed()?;
        let hierarchy = self.speedrun_authored_hierarchy(deck_id);
        let mut mastery = self.speedrun_card_mastery(deck_id)?;

        let touched: HashSet<&String> = concept_ids.iter().collect();
        let mut changed: HashSet<String> = HashSet::new();
        for id in &touched {
            if let Some((_cid, entry)) = mastery.get_mut(*id) {
                entry.seen = true;
                changed.insert((*id).clone());
            }
        }

        let mut upgraded: Vec<String> = Vec::new();
        for node in hierarchy.concept_leaves() {
            let ids = node_concept_ids(node);
            if ids.is_empty() || !ids.iter().any(|id| touched.contains(id)) {
                continue;
            }
            if !ids
                .iter()
                .all(|id| mastery.get(id).map(|(_, e)| e.seen).unwrap_or(false))
            {
                continue;
            }
            for id in &ids {
                if let Some((_cid, entry)) = mastery.get_mut(id) {
                    if entry.state == TopicState::Learning {
                        entry.state = TopicState::Practicing;
                        upgraded.push(id.clone());
                        changed.insert(id.clone());
                    }
                }
            }
        }

        let to_write: Vec<(CardId, ConceptEntry)> = changed
            .iter()
            .filter_map(|id| mastery.get(id).map(|(cid, entry)| (*cid, entry.clone())))
            .collect();
        self.speedrun_persist_entries(&to_write)?;

        if upgraded.is_empty() {
            Ok(json!({
                "upgraded": false,
                "from": TopicState::Learning.as_str(),
                "to": TopicState::Learning.as_str(),
                "conceptIds": [],
            }))
        } else {
            Ok(json!({
                "upgraded": true,
                "from": TopicState::Learning.as_str(),
                "to": TopicState::Practicing.as_str(),
                "conceptIds": upgraded,
            }))
        }
    }

    /// Grade a concept card through real FSRS ([`Collection::grade_now`]) and
    /// move its mastery state. `rating` 1..4 = Again/Hard/Good/Easy. Returns
    /// `{ state, upgraded, from, to, intervalSecs, intervalText }` — the last
    /// two are the next interval this rating schedules, so the grading UI can
    /// show it (omitted if the scheduler couldn't cheaply surface it).
    pub(crate) fn speedrun_answer_card(
        &mut self,
        deck_id: DeckId,
        card_id: CardId,
        concept_id: &str,
        rating: i32,
    ) -> Result<Value> {
        // Read the interval this rating will schedule *before* grading: it comes
        // from the same scheduling states grade_now applies (fuzz is seeded per
        // card, so the two reads agree), and `.ok()` falls back to rating-only.
        let interval = self.speedrun_next_interval(card_id, rating).ok();
        // grade_now uses 0..3 (Again..Easy); the screen sends 1..4.
        self.grade_now(&[card_id], rating - 1)?;
        let mut result = self.speedrun_record_concept_answer(deck_id, concept_id, rating)?;
        if let (Some((secs, text)), Some(obj)) = (interval, result.as_object_mut()) {
            obj.insert("intervalSecs".to_string(), json!(secs));
            obj.insert("intervalText".to_string(), json!(text));
        }
        Ok(result)
    }

    /// The next interval `rating` (1..4 = Again/Hard/Good/Easy) would schedule
    /// for a card, as `(seconds, human string)`. Read from the same scheduling
    /// states [`Collection::grade_now`] applies and formatted with the same
    /// helper as Anki's answer buttons (e.g. `"10m"`, `"4d"`, `"2mo"`).
    fn speedrun_next_interval(&mut self, card_id: CardId, rating: i32) -> Result<(u32, String)> {
        let states = self.get_scheduling_states(card_id)?;
        let state = match rating {
            RATING_AGAIN => states.again,
            2 => states.hard,
            RATING_GOOD => states.good,
            4 => states.easy,
            _ => states.good,
        };
        let now = TimestampSecs::now();
        let timing = self.timing_for_timestamp(now)?;
        let secs_until_rollover = timing.next_day_at.elapsed_secs_since(now).max(0) as u32;
        let secs = state
            .interval_kind()
            .maybe_as_days(secs_until_rollover)
            .as_seconds();
        let text = answer_button_time_collapsible(secs, self.learn_ahead_secs(), &self.tr);
        Ok((secs, text))
    }

    /// Record a difficulty rating against a concept and move its mastery state
    /// on the concept's card (FSRS timing is handled separately by
    /// [`Collection::grade_now`] in [`Collection::speedrun_answer_card`]).
    /// `Again` demotes one state (floor `practicing`); any other rating
    /// advances one state once the rolling signal clears. Returns
    /// `{ state, upgraded, from, to }`.
    fn speedrun_record_concept_answer(
        &mut self,
        deck_id: DeckId,
        concept_id: &str,
        rating: i32,
    ) -> Result<Value> {
        self.speedrun_migrate_progress_if_needed()?;
        let mut mastery = self.speedrun_card_mastery(deck_id)?;
        // The concept is graded via a materialized card, so it is present; a
        // stray id with no card falls back to a default entry and simply has
        // nowhere to persist.
        let (card_id, mut entry) = match mastery.remove(concept_id) {
            Some((cid, entry)) => (Some(cid), entry),
            None => (None, ConceptEntry::default()),
        };

        // Answers only reach concepts at practicing or above; floor the
        // effective pre-answer state so a stray learning concept never drops
        // below practicing.
        let mut current = entry.state;
        if ladder_index(current) < PRACTICING_INDEX {
            current = TopicState::Practicing;
        }

        // Performance evidence: the Applying/Mastering stages render an
        // application problem, so grading one there is an application attempt.
        // A ≥Good rating counts as getting the problem right (same threshold
        // the reviewer uses). Practicing/learning are recall, not application,
        // so they never move Performance.
        if matches!(current, TopicState::Hierarchy | TopicState::Mastering) {
            entry.app_total += 1;
            if rating >= RATING_GOOD {
                entry.app_correct += 1;
            }
        }

        entry.ratings.push(rating);
        if entry.ratings.len() > MASTERY_REVIEW_WINDOW {
            let excess = entry.ratings.len() - MASTERY_REVIEW_WINDOW;
            entry.ratings.drain(0..excess);
        }
        entry.seen = true;

        let new_state = if rating == RATING_AGAIN {
            demoted(current)
        } else if signal_cleared(&entry.ratings) {
            advanced(current)
        } else {
            current
        };
        entry.state = new_state;
        if let Some(card_id) = card_id {
            self.speedrun_persist_entries(&[(card_id, entry)])?;
        }

        Ok(json!({
            "state": new_state.as_str(),
            "upgraded": ladder_index(new_state) > ladder_index(current),
            "from": current.as_str(),
            "to": new_state.as_str(),
        }))
    }

    /// The authored hierarchy blob for a deck (the screen renders concept
    /// content/problems from it), or a fresh empty one seeded with the deck's
    /// name so the screen always has a root title. Mirrors the desktop
    /// `authoring.get_hierarchy` read.
    pub(crate) fn speedrun_study_hierarchy(&mut self, deck_id: DeckId) -> Result<Value> {
        let key = deck_id.0.to_string();
        if let Some(blob) = self.speedrun_authoring_store().get(&key) {
            return Ok(blob.clone());
        }
        let title = self
            .get_deck(deck_id)?
            .map(|d| d.human_name())
            .unwrap_or_default();
        Ok(json!({
            "deckId": key,
            "root": { "id": "root", "title": title, "children": [], "concepts": [] },
        }))
    }

    // --- materialization ---------------------------------------------------

    /// Ensure exactly one [`ITEM_NOTETYPE_NAME`] card per authored concept in
    /// the deck (create missing by `ConceptId`, remove orphaned or duplicate),
    /// and enable FSRS. Idempotent: an unchanged hierarchy is a no-op. This is
    /// what gives the authored concepts genuine FSRS scheduling.
    pub(crate) fn speedrun_reconcile(&mut self, deck_id: DeckId) -> Result<ReconcileOutcome> {
        let hierarchy = self.speedrun_authored_hierarchy(deck_id);
        // conceptId -> title, first occurrence wins, deterministic order.
        let mut wanted: Vec<(String, String)> = Vec::new();
        let mut wanted_ids: HashSet<String> = HashSet::new();
        for node in hierarchy.all_nodes() {
            for concept in &node.concepts {
                if !concept.id.is_empty() && wanted_ids.insert(concept.id.clone()) {
                    wanted.push((concept.id.clone(), concept.title.clone()));
                }
            }
        }

        let notetype_id = self.speedrun_install_item_notetype()?;

        let mut kept: HashSet<String> = HashSet::new();
        let mut orphans: Vec<NoteId> = Vec::new();
        for cid in self.search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)? {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            if note.notetype_id != notetype_id {
                continue;
            }
            let concept_id = note
                .fields()
                .get(ITEM_FIELD_CONCEPT_ID)
                .cloned()
                .unwrap_or_default();
            if wanted_ids.contains(&concept_id) && kept.insert(concept_id) {
                // kept
            } else {
                orphans.push(note.id);
            }
        }

        let notetype = self.get_notetype(notetype_id)?.or_not_found(notetype_id)?;
        let mut created = 0;
        for (concept_id, title) in &wanted {
            if kept.contains(concept_id) {
                continue;
            }
            let mut note = notetype.new_note();
            note.set_field(ITEM_FIELD_CONCEPT_ID, concept_id.as_str())?;
            note.set_field(ITEM_FIELD_TITLE, title.as_str())?;
            self.add_note(&mut note, deck_id)?;
            created += 1;
        }

        if !orphans.is_empty() {
            self.remove_notes(&orphans)?;
        }

        if !self.get_config_bool(BoolKey::Fsrs) {
            self.set_config_bool(BoolKey::Fsrs, true, false)?;
        }

        Ok(ReconcileOutcome {
            created,
            removed: orphans.len(),
            total: wanted.len(),
        })
    }

    /// The `ConceptId` a note maps to, or `""` for any non-`SpeedrunItem` note.
    fn speedrun_concept_id_for_note(&mut self, note: &Note) -> Result<String> {
        let item_id = self
            .get_notetype_by_name(ITEM_NOTETYPE_NAME)?
            .map(|nt| nt.id);
        if Some(note.notetype_id) != item_id {
            return Ok(String::new());
        }
        Ok(note
            .fields()
            .get(ITEM_FIELD_CONCEPT_ID)
            .cloned()
            .unwrap_or_default())
    }

    /// Return the `SpeedrunItem` note type id, creating it if absent.
    fn speedrun_install_item_notetype(&mut self) -> Result<NotetypeId> {
        if let Some(nt) = self.get_notetype_by_name(ITEM_NOTETYPE_NAME)? {
            return Ok(nt.id);
        }
        let mut nt = Notetype {
            name: ITEM_NOTETYPE_NAME.to_string(),
            ..Default::default()
        };
        nt.add_field("ConceptId");
        nt.add_field("Title");
        nt.add_template("Item", ITEM_QFMT, ITEM_AFMT);
        self.add_notetype(&mut nt, true)?;
        Ok(nt.id)
    }

    // --- authored-engine views for the scores ------------------------------

    /// Every authored leaf topic (a node holding concepts) in tree order, with
    /// its display path and concept ids. The single source of truth the
    /// authored Memory/Performance scores and the per-subject breakdown roll
    /// up.
    pub(crate) fn speedrun_authored_leaves(&self, deck_id: DeckId) -> Vec<AuthoredLeaf> {
        let hierarchy = self.speedrun_authored_hierarchy(deck_id);
        let mut out = Vec::new();
        let Some(root) = &hierarchy.root else {
            return out;
        };
        // Concepts attached straight to the root are their own leaf.
        if !root.concepts.is_empty() {
            out.push(AuthoredLeaf {
                node_id: root.id.clone(),
                title: root.title.clone(),
                path: vec![root.title.clone()],
                concept_ids: node_concept_ids(root),
            });
        }
        for child in &root.children {
            collect_authored_leaves(child, &[], &mut out);
        }
        out
    }

    /// Per-concept FSRS evidence over the deck's materialized `SpeedrunItem`
    /// cards: `conceptId -> {retrievability, graded_reviews}`. The Memory score
    /// and breakdown read recall straight off these cards (one per concept),
    /// so studying the authored screen moves the score. Empty when the note
    /// type has never been installed (nothing materialized yet).
    pub(crate) fn speedrun_concept_card_stats(
        &mut self,
        deck_id: DeckId,
    ) -> Result<HashMap<String, ConceptCardStat>> {
        let Some(notetype_id) = self
            .get_notetype_by_name(ITEM_NOTETYPE_NAME)?
            .map(|nt| nt.id)
        else {
            return Ok(HashMap::new());
        };
        let timing = self.timing_today()?;
        let fsrs = FSRS::new(None)?;
        let mut out = HashMap::new();
        for cid in self.search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)? {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            if note.notetype_id != notetype_id {
                continue;
            }
            let concept_id = note
                .fields()
                .get(ITEM_FIELD_CONCEPT_ID)
                .cloned()
                .unwrap_or_default();
            if concept_id.is_empty() {
                continue;
            }
            let graded_reviews = self
                .storage
                .get_revlog_entries_for_card(cid)?
                .iter()
                .filter(|entry| entry.has_rating_and_affects_scheduling())
                .count() as u32;
            let retrievability = card_retrievability(&card, &timing, &fsrs);
            out.insert(
                concept_id,
                ConceptCardStat {
                    retrievability,
                    graded_reviews,
                },
            );
        }
        Ok(out)
    }

    /// Directly seed a concept's application-attempt counts, for the score
    /// tests. Real code only ever writes these through
    /// [`Collection::speedrun_record_concept_answer`]; this bypass lets a test
    /// build an arbitrary accuracy record without fighting the state machine.
    #[cfg(test)]
    pub(crate) fn speedrun_seed_app_counts(
        &mut self,
        deck_id: DeckId,
        concept_id: &str,
        correct: u32,
        total: u32,
    ) {
        self.speedrun_reconcile(deck_id).unwrap();
        let mut mastery = self.speedrun_card_mastery(deck_id).unwrap();
        let (card_id, mut entry) = mastery.remove(concept_id).unwrap();
        entry.seen = true;
        entry.app_correct = correct;
        entry.app_total = total;
        self.speedrun_persist_entries(&[(card_id, entry)]).unwrap();
    }
}

/// One authored leaf topic (a node that directly holds concepts) with its
/// display path and concept ids, for the authored-engine scores.
pub(crate) struct AuthoredLeaf {
    pub node_id: String,
    pub title: String,
    /// Ancestor titles from the top category down to and including this leaf,
    /// root excluded, e.g. `["Enzymes", "Kinetics"]`.
    pub path: Vec<String>,
    pub concept_ids: Vec<String>,
}

/// FSRS evidence for one concept's materialized `SpeedrunItem` card.
pub(crate) struct ConceptCardStat {
    /// Current FSRS retrievability in `[0, 1]` (the 0.9 no-memory prior applies
    /// to a never-graded card).
    pub retrievability: f32,
    /// Scheduling-affecting revlog entries over the card.
    pub graded_reviews: u32,
}

/// Depth-first collect of every concept-bearing node under `node`, threading
/// the ancestor title path (root excluded by the caller).
fn collect_authored_leaves(node: &AuthoredNode, prefix: &[String], out: &mut Vec<AuthoredLeaf>) {
    let mut path = prefix.to_vec();
    path.push(node.title.clone());
    if !node.concepts.is_empty() {
        out.push(AuthoredLeaf {
            node_id: node.id.clone(),
            title: node.title.clone(),
            path: path.clone(),
            concept_ids: node_concept_ids(node),
        });
    }
    for child in &node.children {
        collect_authored_leaves(child, &path, out);
    }
}

/// Shared test fixtures for the authored engine, reused by the score modules'
/// tests so they build the same deck shape this module does.
#[cfg(test)]
pub(crate) mod testing {
    use std::collections::HashMap;

    use serde_json::json;
    use serde_json::Value;

    use super::AUTHORING_CONFIG_KEY;
    use super::ITEM_FIELD_CONCEPT_ID;
    use super::ITEM_NOTETYPE_NAME;
    use crate::prelude::*;
    use crate::search::SearchNode;
    use crate::search::SortMode;

    pub(crate) fn concept(id: &str) -> Value {
        json!({
            "id": id,
            "title": id.to_uppercase(),
            "content": format!("about {id}"),
            "problems": [],
        })
    }

    /// Store an authoring blob whose root has one child node per leaf title,
    /// each holding the given concept ids (mirrors the Python test fixture).
    pub(crate) fn set_hierarchy(col: &mut Collection, deck_id: &str, leaves: &[(&str, &[&str])]) {
        let children: Vec<Value> = leaves
            .iter()
            .map(|(title, cids)| {
                json!({
                    "id": format!("node-{title}"),
                    "title": title,
                    "children": [],
                    "concepts": cids.iter().map(|c| concept(c)).collect::<Vec<_>>(),
                })
            })
            .collect();
        let blob = json!({
            "deckId": deck_id,
            "root": { "id": "root", "title": "Biochem", "concepts": [], "children": children },
        });
        col.set_config(AUTHORING_CONFIG_KEY, &json!({ deck_id: blob }))
            .unwrap();
    }

    /// conceptId -> cardId for every SpeedrunItem card in the deck.
    pub(crate) fn item_cards(col: &mut Collection, deck_id: DeckId) -> HashMap<String, CardId> {
        let item_id = col
            .get_notetype_by_name(ITEM_NOTETYPE_NAME)
            .unwrap()
            .map(|nt| nt.id);
        let mut out = HashMap::new();
        for cid in col
            .search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)
            .unwrap()
        {
            let card = col.storage.get_card(cid).unwrap().unwrap();
            let note = col.storage.get_note(card.note_id).unwrap().unwrap();
            if Some(note.notetype_id) == item_id {
                out.insert(note.fields()[ITEM_FIELD_CONCEPT_ID].clone(), cid);
            }
        }
        out
    }

    /// Reconcile the deck then grade one concept's materialized card with
    /// `rating` (1..4 = Again/Hard/Good/Easy), the same path the study screen
    /// drives. Advances the concept's mastery state as a real answer would.
    pub(crate) fn grade_concept(
        col: &mut Collection,
        deck_id: DeckId,
        concept_id: &str,
        rating: i32,
    ) {
        col.speedrun_reconcile(deck_id).unwrap();
        let card_id = item_cards(col, deck_id)[concept_id];
        col.speedrun_answer_card(deck_id, card_id, concept_id, rating)
            .unwrap();
    }

    /// Mark concepts learned (flips a fully-seen leaf learning -> practicing).
    pub(crate) fn learn_leaf(col: &mut Collection, deck_id: DeckId, concept_ids: &[&str]) {
        let ids: Vec<String> = concept_ids.iter().map(|s| s.to_string()).collect();
        col.speedrun_record_learned(deck_id, &ids).unwrap();
    }

    /// Drive a concept up to the Applying stage (`hierarchy`): learn its whole
    /// leaf, then two Goods to clear the practicing -> hierarchy signal.
    pub(crate) fn advance_to_applying(
        col: &mut Collection,
        deck_id: DeckId,
        leaf_concept_ids: &[&str],
        concept_id: &str,
    ) {
        learn_leaf(col, deck_id, leaf_concept_ids);
        grade_concept(col, deck_id, concept_id, 3);
        grade_concept(col, deck_id, concept_id, 3);
    }

    /// Drive a concept to Applying, then grade it Good `count` times through
    /// the real answer path. Every grade at Applying/Mastering is a
    /// *correct* application attempt (Good never leaves the application
    /// stages), so this deterministically records `count` correct attempts.
    /// For mixed accuracy a test seeds counts directly with
    /// [`Collection::speedrun_seed_app_counts`].
    pub(crate) fn apply_correct_attempts(
        col: &mut Collection,
        deck_id: DeckId,
        leaf_concept_ids: &[&str],
        concept_id: &str,
        count: u32,
    ) {
        advance_to_applying(col, deck_id, leaf_concept_ids, concept_id);
        for _ in 0..count {
            grade_concept(col, deck_id, concept_id, 3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::advance_to_applying;
    use super::testing::grade_concept;
    use super::testing::item_cards;
    use super::testing::set_hierarchy;
    use super::*;

    fn progress_of(col: &mut Collection, deck_id: DeckId) -> Value {
        col.speedrun_study_state(deck_id).unwrap()["progress"].clone()
    }

    // --- initial state -----------------------------------------------------

    #[test]
    fn absent_concept_defaults_to_learning_unseen() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1", "c2"])]);
        let progress = progress_of(&mut col, DeckId(1));
        assert_eq!(
            progress["c1"],
            json!({ "state": "learning", "seen": false })
        );
        assert_eq!(
            progress["c2"],
            json!({ "state": "learning", "seen": false })
        );
    }

    // --- topic-gated learning flip (ST7) -----------------------------------

    #[test]
    fn record_learned_is_topic_gated() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1", "c2"])]);

        // Learning one of two concepts marks it seen but does not flip the topic.
        let first = col
            .speedrun_record_learned(DeckId(1), &["c1".to_string()])
            .unwrap();
        assert_eq!(first["upgraded"], json!(false));
        assert_eq!(first["conceptIds"], json!([]));
        let progress = progress_of(&mut col, DeckId(1));
        assert_eq!(progress["c1"], json!({ "state": "learning", "seen": true }));
        assert_eq!(
            progress["c2"],
            json!({ "state": "learning", "seen": false })
        );

        // Learning the last concept flips the whole leaf together.
        let second = col
            .speedrun_record_learned(DeckId(1), &["c2".to_string()])
            .unwrap();
        assert_eq!(second["upgraded"], json!(true));
        assert_eq!(second["from"], json!("learning"));
        assert_eq!(second["to"], json!("practicing"));
        let flipped: HashSet<String> =
            serde_json::from_value(second["conceptIds"].clone()).unwrap();
        assert_eq!(flipped, HashSet::from(["c1".to_string(), "c2".to_string()]));
        let progress = progress_of(&mut col, DeckId(1));
        assert_eq!(progress["c1"]["state"], json!("practicing"));
        assert_eq!(progress["c2"]["state"], json!("practicing"));
    }

    #[test]
    fn record_learned_only_flips_the_completed_leaf() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[("Structure", &["a1"]), ("Kinetics", &["b1", "b2"])],
        );

        let result = col
            .speedrun_record_learned(DeckId(1), &["a1".to_string()])
            .unwrap();
        assert_eq!(result["upgraded"], json!(true));
        assert_eq!(result["conceptIds"], json!(["a1"]));
        let progress = progress_of(&mut col, DeckId(1));
        assert_eq!(progress["a1"]["state"], json!("practicing"));
        assert_eq!(progress["b1"]["state"], json!("learning"));
        assert_eq!(progress["b2"]["state"], json!("learning"));
    }

    #[test]
    fn next_card_reports_the_first_unfinished_topic_then_reviews() {
        let mut col = Collection::new();
        set_hierarchy(
            &mut col,
            "1",
            &[("Structure", &["a1"]), ("Kinetics", &["b1", "b2"])],
        );
        col.speedrun_record_learned(DeckId(1), &["a1".to_string()])
            .unwrap();

        let block = col.speedrun_next_card(DeckId(1)).unwrap();
        assert_eq!(block["kind"], json!("learning_block"));
        assert_eq!(block["topicNodeId"], json!("node-Kinetics"));
        assert_eq!(block["conceptIds"], json!(["b1", "b2"]));
        assert_eq!(block["learnedCount"], json!(0));
        assert_eq!(block["totalCount"], json!(2));

        // Once every concept is learned there is no learning block left, so the
        // scheduler serves a materialized review card instead.
        col.speedrun_record_learned(DeckId(1), &["b1".to_string(), "b2".to_string()])
            .unwrap();
        let next = col.speedrun_next_card(DeckId(1)).unwrap();
        assert_eq!(next["kind"], json!("review"));
        // The served card maps back to one of the authored concepts.
        let served = next["conceptId"].as_str().unwrap().to_string();
        assert!(["a1", "b1", "b2"].contains(&served.as_str()));
    }

    // --- per-concept advance / demote (ST8) --------------------------------

    fn practice(col: &mut Collection, deck_id: DeckId, concept_ids: &[&str]) {
        let ids: Vec<String> = concept_ids.iter().map(|s| s.to_string()).collect();
        col.speedrun_record_learned(deck_id, &ids).unwrap();
    }

    #[test]
    fn record_answer_advances_once_signal_clears() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1", "c2"])]);
        practice(&mut col, DeckId(1), &["c1", "c2"]);

        // One Good is below MIN_REPS, so the concept stays practicing.
        let first = col
            .speedrun_record_concept_answer(DeckId(1), "c1", RATING_GOOD)
            .unwrap();
        assert_eq!(
            first,
            json!({ "state": "practicing", "upgraded": false, "from": "practicing", "to": "practicing" })
        );

        // A second Good clears the signal and advances one state (Applying).
        let second = col
            .speedrun_record_concept_answer(DeckId(1), "c1", RATING_GOOD)
            .unwrap();
        assert_eq!(second["upgraded"], json!(true));
        assert_eq!(second["from"], json!("practicing"));
        assert_eq!(second["to"], json!("hierarchy"));
        assert_eq!(second["state"], json!("hierarchy"));
    }

    #[test]
    fn record_answer_demotes_on_again() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1", "c2"])]);
        practice(&mut col, DeckId(1), &["c1", "c2"]);
        col.speedrun_record_concept_answer(DeckId(1), "c1", RATING_GOOD)
            .unwrap();
        col.speedrun_record_concept_answer(DeckId(1), "c1", RATING_GOOD)
            .unwrap();
        assert_eq!(
            progress_of(&mut col, DeckId(1))["c1"]["state"],
            json!("hierarchy")
        );

        let result = col
            .speedrun_record_concept_answer(DeckId(1), "c1", RATING_AGAIN)
            .unwrap();
        assert_eq!(result["from"], json!("hierarchy"));
        assert_eq!(result["to"], json!("practicing"));
        assert_eq!(result["upgraded"], json!(false));
        assert_eq!(result["state"], json!("practicing"));
    }

    #[test]
    fn record_answer_floors_at_practicing() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1", "c2"])]);
        practice(&mut col, DeckId(1), &["c1", "c2"]);

        let result = col
            .speedrun_record_concept_answer(DeckId(1), "c1", RATING_AGAIN)
            .unwrap();
        assert_eq!(result["state"], json!("practicing"));
        assert_eq!(result["to"], json!("practicing"));
        assert_eq!(
            progress_of(&mut col, DeckId(1))["c1"]["state"],
            json!("practicing")
        );
    }

    // --- materialize / reconcile (ST12) ------------------------------------

    #[test]
    fn reconcile_creates_one_card_per_concept_and_enables_fsrs() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1", "c2"])]);

        let result = col.speedrun_reconcile(DeckId(1)).unwrap();
        assert_eq!(
            result,
            ReconcileOutcome {
                created: 2,
                removed: 0,
                total: 2
            }
        );

        let cards = item_cards(&mut col, DeckId(1));
        assert_eq!(
            cards.keys().cloned().collect::<HashSet<_>>(),
            HashSet::from(["c1".to_string(), "c2".to_string()])
        );
        assert!(col.get_config_bool(BoolKey::Fsrs), "FSRS enabled");

        // Idempotent: a second reconcile over the same tree changes nothing.
        assert_eq!(
            col.speedrun_reconcile(DeckId(1)).unwrap(),
            ReconcileOutcome {
                created: 0,
                removed: 0,
                total: 2
            }
        );
        assert_eq!(item_cards(&mut col, DeckId(1)).len(), 2);
    }

    #[test]
    fn reconcile_removes_orphaned_cards() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1", "c2"])]);
        col.speedrun_reconcile(DeckId(1)).unwrap();
        assert_eq!(item_cards(&mut col, DeckId(1)).len(), 2);

        // Drop c2 from the authored tree; reconcile must remove its card.
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1"])]);
        let result = col.speedrun_reconcile(DeckId(1)).unwrap();
        assert_eq!(
            result,
            ReconcileOutcome {
                created: 0,
                removed: 1,
                total: 1
            }
        );
        assert_eq!(
            item_cards(&mut col, DeckId(1))
                .keys()
                .cloned()
                .collect::<HashSet<_>>(),
            HashSet::from(["c1".to_string()])
        );
    }

    #[test]
    fn answer_card_grades_through_fsrs_and_moves_state() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1"])]);
        practice(&mut col, DeckId(1), &["c1"]);
        col.speedrun_reconcile(DeckId(1)).unwrap();
        let card_id = item_cards(&mut col, DeckId(1))["c1"];

        let revlog_before = col
            .storage
            .get_all_revlog_entries(TimestampSecs(0))
            .unwrap()
            .len();
        let result = col
            .speedrun_answer_card(DeckId(1), card_id, "c1", RATING_GOOD)
            .unwrap();
        assert_eq!(result["state"], json!("practicing"));

        // Real FSRS: one revlog entry written and the card left the New state
        // with an FSRS memory state assigned.
        assert_eq!(
            col.storage
                .get_all_revlog_entries(TimestampSecs(0))
                .unwrap()
                .len(),
            revlog_before + 1
        );
        let graded = col.storage.get_card(card_id).unwrap().unwrap();
        assert_ne!(graded.ctype, crate::card::CardType::New);
        assert!(graded.memory_state.is_some(), "FSRS memory state assigned");
    }

    #[test]
    fn answer_at_applying_records_application_attempt() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["c1"])]);
        // learn + two Goods reaches Applying (hierarchy); both were at
        // practicing, so no application attempt is recorded yet.
        advance_to_applying(&mut col, DeckId(1), &["c1"], "c1");
        assert_eq!(
            col.speedrun_deck_progress(DeckId(1)).unwrap()["c1"].app_total,
            0
        );

        // A Good at the Applying stage is a correct application attempt.
        grade_concept(&mut col, DeckId(1), "c1", 3);
        let entry = col.speedrun_deck_progress(DeckId(1)).unwrap()["c1"].clone();
        assert_eq!(entry.app_total, 1);
        assert_eq!(entry.app_correct, 1);

        // An Again at the application stage is a wrong attempt: counted, then
        // the concept demotes.
        grade_concept(&mut col, DeckId(1), "c1", 1);
        let entry = col.speedrun_deck_progress(DeckId(1)).unwrap()["c1"].clone();
        assert_eq!(entry.app_total, 2);
        assert_eq!(entry.app_correct, 1);
    }

    #[test]
    fn study_hierarchy_returns_stored_blob_and_empty_fallback() {
        let mut col = Collection::new();
        // A deck with no authored blob gets a fresh empty hierarchy seeded with
        // the deck's name.
        let empty = col.speedrun_study_hierarchy(DeckId(1)).unwrap();
        assert_eq!(empty["deckId"], json!("1"));
        assert_eq!(empty["root"]["concepts"], json!([]));

        set_hierarchy(&mut col, "1", &[("Amino acids", &["c1"])]);
        let stored = col.speedrun_study_hierarchy(DeckId(1)).unwrap();
        assert_eq!(stored["root"]["children"][0]["title"], json!("Amino acids"));
        assert_eq!(
            stored["root"]["children"][0]["concepts"][0]["id"],
            json!("c1")
        );
    }

    // --- per-card mastery storage (the sync-mergeable move) ----------------

    /// The concept's card carries a valid, budget-respecting mastery record in
    /// its `custom_data`, and reading it back reproduces the entry.
    #[test]
    fn mastery_is_persisted_on_the_concept_card() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["c1"])]);
        advance_to_applying(&mut col, DeckId(1), &["c1"], "c1");
        grade_concept(&mut col, DeckId(1), "c1", 3);

        let card_id = item_cards(&mut col, DeckId(1))["c1"];
        let card = col.storage.get_card(card_id).unwrap().unwrap();
        // Stored on the card, not the config blob.
        assert!(card.custom_data.contains(CD_STATE));
        assert!(card.custom_data.contains(CD_RATINGS));
        // Stays inside the custom_data budget (<=100 bytes, keys <=8 bytes).
        card.validate_custom_data().unwrap();

        let entry = ConceptEntry::from_custom_data(&card.custom_data);
        // advance_to_applying reaches Applying (hierarchy); the extra Good at the
        // application stage records an attempt and advances to Mastering.
        assert_eq!(entry.state, TopicState::Mastering);
        assert!(entry.seen);
        assert_eq!(entry.app_total, 1);
        assert_eq!(entry.app_correct, 1);
    }

    /// Even a heavily-drilled concept (full ratings window, many attempts)
    /// keeps its `custom_data` within Anki's 100-byte budget.
    #[test]
    fn mastery_custom_data_stays_within_budget() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["c1"])]);
        practice(&mut col, DeckId(1), &["c1"]);
        for _ in 0..60 {
            grade_concept(&mut col, DeckId(1), "c1", 3);
        }
        let card_id = item_cards(&mut col, DeckId(1))["c1"];
        let card = col.storage.get_card(card_id).unwrap().unwrap();
        assert!(
            card.custom_data.len() <= 100,
            "custom_data must stay within budget, got {} bytes: {}",
            card.custom_data.len(),
            card.custom_data
        );
        card.validate_custom_data().unwrap();
    }

    /// Editing one concept's mastery leaves its neighbours' cards untouched.
    /// This per-card isolation is what lets sync merge concurrent offline edits
    /// to different concepts without clobbering (the whole point of the move
    /// off the single config blob).
    #[test]
    fn mastery_edits_are_isolated_per_card() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["c1", "c2"])]);
        practice(&mut col, DeckId(1), &["c1", "c2"]);

        let c2_card = item_cards(&mut col, DeckId(1))["c2"];
        let before = col.storage.get_card(c2_card).unwrap().unwrap();

        // Hammer c1 through several answers.
        for _ in 0..5 {
            grade_concept(&mut col, DeckId(1), "c1", 3);
        }

        let after = col.storage.get_card(c2_card).unwrap().unwrap();
        assert_eq!(
            before.custom_data, after.custom_data,
            "c2's mastery must not change when only c1 is answered"
        );
        assert_eq!(
            before.mtime, after.mtime,
            "c2's card must not be re-touched"
        );
    }

    // --- one-time legacy-blob migration ------------------------------------

    /// A pre-existing `speedrun_study_progress` config blob is folded onto the
    /// concept cards on first read, and drives the scores from there.
    #[test]
    fn legacy_config_progress_migrates_onto_cards() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["c1", "c2"])]);
        // Simulate an old collection whose progress only lives in the blob.
        col.set_config(
            STUDY_PROGRESS_CONFIG_KEY,
            &json!({
                "1": {
                    "c1": { "state": "hierarchy", "seen": true, "ratings": [3, 3],
                            "appCorrect": 3, "appTotal": 4 },
                    "c2": { "state": "practicing", "seen": true, "ratings": [3] },
                }
            }),
        )
        .unwrap();
        assert!(
            !col.get_config_optional::<bool, _>(MASTERY_MIGRATED_CONFIG_KEY)
                .unwrap_or(false),
            "not migrated yet"
        );

        // First read triggers the migration.
        let progress = col.speedrun_deck_progress(DeckId(1)).unwrap();
        assert_eq!(progress["c1"].state, TopicState::Hierarchy);
        assert_eq!(progress["c1"].app_correct, 3);
        assert_eq!(progress["c1"].app_total, 4);
        assert_eq!(progress["c2"].state, TopicState::Practicing);
        assert!(col
            .get_config_optional::<bool, _>(MASTERY_MIGRATED_CONFIG_KEY)
            .unwrap());

        // The migrated evidence now lands on the card, so Performance reads it.
        let perf = col.get_performance_score(DeckId(1)).unwrap();
        assert_eq!(
            perf.graded_reviews, 4,
            "the migrated attempts feed the score"
        );
    }

    /// The migration runs once: a later change to the (retained) legacy blob is
    /// not re-applied, so it can't overwrite fresh per-card progress.
    #[test]
    fn migration_is_one_shot() {
        let mut col = Collection::new();
        set_hierarchy(&mut col, "1", &[("Kinetics", &["c1"])]);
        col.set_config(
            STUDY_PROGRESS_CONFIG_KEY,
            &json!({ "1": { "c1": { "state": "practicing", "seen": true } } }),
        )
        .unwrap();
        // First read migrates c1 -> practicing.
        assert_eq!(
            col.speedrun_deck_progress(DeckId(1)).unwrap()["c1"].state,
            TopicState::Practicing
        );

        // A stale later blob write must NOT be re-migrated over the card.
        col.set_config(
            STUDY_PROGRESS_CONFIG_KEY,
            &json!({ "1": { "c1": { "state": "mastering", "seen": true } } }),
        )
        .unwrap();
        assert_eq!(
            col.speedrun_deck_progress(DeckId(1)).unwrap()["c1"].state,
            TopicState::Practicing,
            "the one-shot guard blocks re-migration"
        );
    }
}
