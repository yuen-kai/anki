// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Per-topic mastery progression — Speedrun Phase 5, decisions D30, D31, D32
//! (see docs/plan/spec-mastery-progression.md, docs/plan/decisions.md).
//!
//! Each AAMC hierarchy leaf topic walks a four-state lifecycle:
//!
//! ```text
//! learning → practicing → hierarchy → mastering
//! ```
//!
//! and each state drives how the topic's cards render ("upgrade the cards to
//! match", spec §2):
//!
//! | State        | `SpeedrunConcept`   | `SpeedrunApplication`     |
//! | :--          | :--                 | :--                       |
//! | learning     | concept_learn       | (suppressed)              |
//! | practicing   | concept_practice    | (suppressed)              |
//! | hierarchy    | concept_practice    | application_scaffolded    |
//! | mastering    | concept_practice    | application_unscaffolded  |
//!
//! The application card is held back until `hierarchy` (no applying before the
//! concept is in hand) and the scaffold is removed at `mastering` (the transfer
//! goal). A non-Speedrun card always resolves to the neutral `none` mode.
//!
//! State lives in the **collection config** as a JSON map `topic_id → {state,
//! updated_at}` under [`TOPIC_STATE_CONFIG_KEY`] (D32): no schema migration, it
//! syncs with the collection, and an absent entry is fail-safe `learning`. A
//! topic **advances** one state when its active-mode cards clear the mastery
//! signal (recent ≥Good accuracy over enough graded reps); an `Again` lapse
//! **demotes** it one state, never below `learning`.
//!
//! Everything here only ever reads card/revlog state and writes the config map
//! — it produces no FSRS interval and never touches the scheduler or the undo
//! queue, so the progression is additive and undo-safe.

use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;
use crate::scheduler::answering::Rating;
use crate::search::SearchNode;
use crate::search::SortMode;
use crate::speedrun::card_signals::card_topic;
use crate::speedrun::card_signals::leaf_topic_weights;
use crate::speedrun::taxonomy::topic_path_labels;

/// Collection-config key holding the per-topic state map (D32).
pub(crate) const TOPIC_STATE_CONFIG_KEY: &str = "speedrun_topic_state";

/// Minimum ≥Good rate over a topic's recent active-mode answers needed to
/// advance one state. Tunable; a guess pending real study data (D32 gap).
pub const ACC_THRESHOLD: f32 = 0.8;
/// Minimum graded answers in the active mode before a topic can advance.
/// Tunable (D32 gap).
pub const MIN_REPS: usize = 2;
/// How many of the active mode's most recent graded answers feed the
/// advancement accuracy signal.
const MASTERY_REVIEW_WINDOW: usize = 50;

/// The two Speedrun note types (must match `pylib/anki/speedrun/notetypes.py`).
const CONCEPT_NOTETYPE_NAME: &str = "SpeedrunConcept";
const APPLICATION_NOTETYPE_NAME: &str = "SpeedrunApplication";

/// The exact card-mode strings the reviewer injects as `window.speedrunCardMode`
/// and the templates render by (spec §5). `none` covers a non-Speedrun card and
/// an application card whose topic is still below `hierarchy` (suppressed).
pub(crate) const MODE_CONCEPT_LEARN: &str = "concept_learn";
pub(crate) const MODE_CONCEPT_PRACTICE: &str = "concept_practice";
pub(crate) const MODE_APPLICATION_SCAFFOLDED: &str = "application_scaffolded";
pub(crate) const MODE_APPLICATION_UNSCAFFOLDED: &str = "application_unscaffolded";
pub(crate) const MODE_NONE: &str = "none";

/// One topic's position in the four-state lifecycle (spec §2). Serialized as a
/// lowercase string in the config map; an absent entry defaults to `Learning`.
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

    /// The next state up the ladder; saturates at `Mastering`.
    fn advanced(self) -> Self {
        match self {
            TopicState::Learning => TopicState::Practicing,
            TopicState::Practicing => TopicState::Hierarchy,
            TopicState::Hierarchy => TopicState::Mastering,
            TopicState::Mastering => TopicState::Mastering,
        }
    }

    /// The next state down the ladder; saturates at `Learning` (D32: a lapse
    /// never drops a topic below the blocked first-exposure state).
    fn demoted(self) -> Self {
        match self {
            TopicState::Learning => TopicState::Learning,
            TopicState::Practicing => TopicState::Learning,
            TopicState::Hierarchy => TopicState::Practicing,
            TopicState::Mastering => TopicState::Hierarchy,
        }
    }

    /// The note kind whose cards are "active" in this state and whose recent
    /// accuracy drives advancement: concept cards while still learning the
    /// concept, application cards once the topic is being applied.
    fn active_note_kind(self) -> NoteKind {
        match self {
            TopicState::Learning | TopicState::Practicing => NoteKind::Concept,
            TopicState::Hierarchy | TopicState::Mastering => NoteKind::Application,
        }
    }
}

/// Which Speedrun note type (if any) a card belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoteKind {
    Concept,
    Application,
    Other,
}

impl NoteKind {
    fn from_notetype_name(name: &str) -> Self {
        match name {
            CONCEPT_NOTETYPE_NAME => NoteKind::Concept,
            APPLICATION_NOTETYPE_NAME => NoteKind::Application,
            _ => NoteKind::Other,
        }
    }

    /// The note-type name to gather this kind's cards by; `None` for `Other`.
    fn notetype_name(self) -> Option<&'static str> {
        match self {
            NoteKind::Concept => Some(CONCEPT_NOTETYPE_NAME),
            NoteKind::Application => Some(APPLICATION_NOTETYPE_NAME),
            NoteKind::Other => None,
        }
    }
}

/// Resolve a card's active mode from its note kind and its topic's state (spec
/// §2/§5). A `SpeedrunApplication` card is suppressed (the neutral `none`)
/// while its topic is below `hierarchy`; any non-Speedrun card is always
/// `none`. Pure: the single source of truth for both the per-card RPC and the
/// queue's suppression.
pub(crate) fn resolve_card_mode(kind: NoteKind, state: TopicState) -> &'static str {
    match kind {
        NoteKind::Concept => match state {
            TopicState::Learning => MODE_CONCEPT_LEARN,
            _ => MODE_CONCEPT_PRACTICE,
        },
        NoteKind::Application => match state {
            TopicState::Hierarchy => MODE_APPLICATION_SCAFFOLDED,
            TopicState::Mastering => MODE_APPLICATION_UNSCAFFOLDED,
            TopicState::Learning | TopicState::Practicing => MODE_NONE,
        },
        NoteKind::Other => MODE_NONE,
    }
}

/// Whether an application card is suppressed (not served) for a topic in
/// `state` — true while the topic is still below `hierarchy` (spec §2/§6). The
/// queue uses this to drop application cards; it mirrors the `Application →
/// none` branch of [`resolve_card_mode`].
pub(crate) fn application_suppressed(state: TopicState) -> bool {
    matches!(state, TopicState::Learning | TopicState::Practicing)
}

/// The state a topic moves to after one answer (spec §3): `Again` demotes a
/// step; any other rating advances a step only once `signal_cleared`. Both
/// directions saturate. Pure, so the transition logic is unit-tested directly.
fn next_topic_state(current: TopicState, rating: Rating, signal_cleared: bool) -> TopicState {
    // Rating derives only Copy/Clone, so match rather than `==`.
    if matches!(rating, Rating::Again) {
        current.demoted()
    } else if signal_cleared {
        current.advanced()
    } else {
        current
    }
}

/// One topic's persisted state plus when it last changed (D32).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TopicStateEntry {
    state: TopicState,
    updated_at: i64,
}

/// `topic_id → {state, updated_at}`; the value stored under
/// [`TOPIC_STATE_CONFIG_KEY`].
type TopicStateMap = HashMap<String, TopicStateEntry>;

impl Collection {
    /// The whole per-topic state map; an absent or unparsable value is treated
    /// as empty (so every topic falls back to `learning`, fail-safe per D32).
    fn speedrun_topic_state_map(&self) -> TopicStateMap {
        self.get_config_optional(TOPIC_STATE_CONFIG_KEY)
            .unwrap_or_default()
    }

    /// The stored state for one topic, defaulting to `learning` when absent.
    pub(crate) fn speedrun_topic_state(&self, topic: &str) -> TopicState {
        self.speedrun_topic_state_map()
            .get(topic)
            .map(|entry| entry.state)
            .unwrap_or_default()
    }

    /// Every topic carrying an explicit state, as `topic_id → state`, so a
    /// caller can resolve many topics with a single config read (topics absent
    /// from the result are `learning`). Used by the state-aware queue.
    pub(crate) fn speedrun_topic_states(&self) -> HashMap<String, TopicState> {
        self.speedrun_topic_state_map()
            .into_iter()
            .map(|(topic, entry)| (topic, entry.state))
            .collect()
    }

    /// Persist one topic's state, stamping `updated_at`. Written non-undoably
    /// (`Op::SkipUndo`) so the config write leaves the card-answer undo entry on
    /// top of the undo stack untouched — the progression never disturbs undo.
    pub(crate) fn set_speedrun_topic_state(
        &mut self,
        topic: &str,
        state: TopicState,
    ) -> Result<()> {
        let mut map = self.speedrun_topic_state_map();
        map.insert(
            topic.to_string(),
            TopicStateEntry {
                state,
                updated_at: TimestampSecs::now().0,
            },
        );
        self.set_config_json(TOPIC_STATE_CONFIG_KEY, &map, false)?;
        Ok(())
    }

    /// Resolve a card's note kind, its taxonomy topic (if any) and that topic's
    /// mastery state in a single lookup — the shared core of both the card-mode
    /// and card-context resolution. A non-Speedrun card short-circuits to
    /// `(Other, None, default)` without a (pointless) topic lookup.
    fn speedrun_card_kind_topic_state(
        &mut self,
        card_id: CardId,
    ) -> Result<(NoteKind, Option<String>, TopicState)> {
        let card = self.storage.get_card(card_id)?.or_not_found(card_id)?;
        let note = self
            .storage
            .get_note(card.note_id)?
            .or_not_found(card.note_id)?;
        let kind = self.note_kind(&note)?;
        if kind == NoteKind::Other {
            return Ok((kind, None, TopicState::default()));
        }
        let leaf_weights = leaf_topic_weights();
        let topic = card_topic(&note.tags, &leaf_weights);
        let state = topic
            .as_deref()
            .map(|topic| self.speedrun_topic_state(topic))
            .unwrap_or_default();
        Ok((kind, topic, state))
    }

    /// The active card mode for a card (spec §5): resolve its note kind and its
    /// topic's state, then map to one of the mode strings. A non-Speedrun card,
    /// or a card with no taxonomy topic, resolves against the default state.
    pub(crate) fn get_speedrun_card_mode(&mut self, card_id: CardId) -> Result<&'static str> {
        let (kind, _topic, state) = self.speedrun_card_kind_topic_state(card_id)?;
        Ok(resolve_card_mode(kind, state))
    }

    /// A card's full render context (spec §5): its active mode plus the display
    /// labels of its taxonomy hierarchy path (foundation → leaf, e.g.
    /// `["Biomolecules", "Enzymes", "Inhibition"]`), so the reviewer can inject
    /// both `window.speedrunCardMode` and `window.speedrunTopicPath` for the
    /// breadcrumb. The path is empty for a non-Speedrun card or one with no
    /// taxonomy topic, which the template renders as no breadcrumb.
    pub(crate) fn get_speedrun_card_context(
        &mut self,
        card_id: CardId,
    ) -> Result<(&'static str, Vec<String>)> {
        let (kind, topic, state) = self.speedrun_card_kind_topic_state(card_id)?;
        let mode = resolve_card_mode(kind, state);
        let path = topic.as_deref().map(topic_path_labels).unwrap_or_default();
        Ok((mode, path))
    }

    /// Record an answer against the card's topic and return the topic's new
    /// state (spec §3, D32). `Again` demotes one state; any other rating
    /// advances one state once the topic's active-mode cards clear the mastery
    /// signal. A card with no taxonomy topic is a no-op (returns `learning`).
    pub(crate) fn speedrun_record_answer(
        &mut self,
        card_id: CardId,
        rating: Rating,
    ) -> Result<TopicState> {
        let card = self.storage.get_card(card_id)?.or_not_found(card_id)?;
        let note = self
            .storage
            .get_note(card.note_id)?
            .or_not_found(card.note_id)?;
        let leaf_weights = leaf_topic_weights();
        let Some(topic) = card_topic(&note.tags, &leaf_weights) else {
            return Ok(TopicState::default());
        };

        let current = self.speedrun_topic_state(&topic);
        // Only the advancement path needs the (costlier) signal lookup.
        let signal_cleared = !matches!(rating, Rating::Again)
            && self.topic_mastery_signal_cleared(&topic, current.active_note_kind())?;
        let next = next_topic_state(current, rating, signal_cleared);
        if next != current {
            self.set_speedrun_topic_state(&topic, next)?;
        }
        Ok(next)
    }

    /// Per-topic progress for a deck's in-scope topics, for the dashboard (spec
    /// §7). Returns `(topic_id, state)` for every in-scope taxonomy topic the
    /// deck has at least one card mapped to, sorted by id for determinism.
    pub(crate) fn get_speedrun_progress(
        &mut self,
        deck_id: DeckId,
    ) -> Result<Vec<(String, TopicState)>> {
        let leaf_weights = leaf_topic_weights();
        let card_ids =
            self.search_cards(SearchNode::from_deck_id(deck_id, true), SortMode::NoOrder)?;
        let mut covered: HashSet<String> = HashSet::new();
        for cid in card_ids {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            if let Some(topic) = card_topic(&note.tags, &leaf_weights) {
                covered.insert(topic);
            }
        }
        let map = self.speedrun_topic_state_map();
        let mut out: Vec<(String, TopicState)> = covered
            .into_iter()
            .map(|topic| {
                let state = map.get(&topic).map(|e| e.state).unwrap_or_default();
                (topic, state)
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
    }

    /// Whether the topic's active-mode cards clear the advancement signal (spec
    /// §3): at least [`MIN_REPS`] recent graded answers over the topic's cards
    /// of `active_kind`, with a ≥Good rate of at least [`ACC_THRESHOLD`].
    fn topic_mastery_signal_cleared(&mut self, topic: &str, active_kind: NoteKind) -> Result<bool> {
        let Some(notetype_name) = active_kind.notetype_name() else {
            return Ok(false);
        };
        let leaf_weights = leaf_topic_weights();
        let card_ids = self.search_cards(
            SearchNode::Notetype(notetype_name.into()),
            SortMode::NoOrder,
        )?;
        let mut entries = Vec::new();
        for cid in card_ids {
            let card = self.storage.get_card(cid)?.or_not_found(cid)?;
            let note = self
                .storage
                .get_note(card.note_id)?
                .or_not_found(card.note_id)?;
            // A note can carry several leaf tags; only count it for the topic
            // card_topic actually resolves it to, matching the queue/score.
            if card_topic(&note.tags, &leaf_weights).as_deref() != Some(topic) {
                continue;
            }
            for entry in self.storage.get_revlog_entries_for_card(cid)? {
                if entry.has_rating_and_affects_scheduling() {
                    entries.push(entry);
                }
            }
        }
        if entries.len() < MIN_REPS {
            return Ok(false);
        }
        // Most recent first (revlog id is a millisecond timestamp), capped.
        entries.sort_by(|a, b| b.id.0.cmp(&a.id.0));
        entries.truncate(MASTERY_REVIEW_WINDOW);
        // Buttons: 1 Again, 2 Hard, 3 Good, 4 Easy — ≥Good (3) counts as a clear.
        let good = entries.iter().filter(|e| e.button_chosen >= 3).count();
        Ok(good as f32 / entries.len() as f32 >= ACC_THRESHOLD)
    }

    /// The [`NoteKind`] of a note, from its note type's name.
    pub(crate) fn note_kind(&mut self, note: &Note) -> Result<NoteKind> {
        Ok(self
            .get_notetype(note.notetype_id)?
            .map(|nt| NoteKind::from_notetype_name(&nt.name))
            .unwrap_or(NoteKind::Other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revlog::RevlogEntry;
    use crate::revlog::RevlogReviewKind;

    const KINETICS: &str = "mcat::biomolecules::enzymes::kinetics";
    const STRUCTURE: &str = "mcat::biomolecules::amino_acids::structure";

    // (a) State transitions — exercised directly on the pure transition fn.

    #[test]
    fn advances_one_state_when_signal_clears() {
        assert_eq!(
            next_topic_state(TopicState::Learning, Rating::Good, true),
            TopicState::Practicing
        );
        assert_eq!(
            next_topic_state(TopicState::Practicing, Rating::Hard, true),
            TopicState::Hierarchy
        );
        assert_eq!(
            next_topic_state(TopicState::Hierarchy, Rating::Easy, true),
            TopicState::Mastering
        );
    }

    #[test]
    fn does_not_advance_until_signal_clears() {
        assert_eq!(
            next_topic_state(TopicState::Learning, Rating::Good, false),
            TopicState::Learning
        );
        assert_eq!(
            next_topic_state(TopicState::Hierarchy, Rating::Easy, false),
            TopicState::Hierarchy
        );
    }

    #[test]
    fn mastering_never_advances_past_itself() {
        assert_eq!(
            next_topic_state(TopicState::Mastering, Rating::Easy, true),
            TopicState::Mastering
        );
    }

    #[test]
    fn again_demotes_one_state_each_step() {
        assert_eq!(
            next_topic_state(TopicState::Mastering, Rating::Again, true),
            TopicState::Hierarchy
        );
        assert_eq!(
            next_topic_state(TopicState::Hierarchy, Rating::Again, false),
            TopicState::Practicing
        );
        assert_eq!(
            next_topic_state(TopicState::Practicing, Rating::Again, false),
            TopicState::Learning
        );
    }

    #[test]
    fn lapse_never_drops_below_learning() {
        assert_eq!(
            next_topic_state(TopicState::Learning, Rating::Again, false),
            TopicState::Learning
        );
    }

    // (b) Card-mode resolution — both note types across every state, incl. none.

    #[test]
    fn concept_modes_across_all_states() {
        assert_eq!(
            resolve_card_mode(NoteKind::Concept, TopicState::Learning),
            MODE_CONCEPT_LEARN
        );
        for state in [
            TopicState::Practicing,
            TopicState::Hierarchy,
            TopicState::Mastering,
        ] {
            assert_eq!(
                resolve_card_mode(NoteKind::Concept, state),
                MODE_CONCEPT_PRACTICE,
                "concept stays in practice mode at {state:?}"
            );
        }
    }

    #[test]
    fn application_modes_across_all_states() {
        // Suppressed (none) below hierarchy.
        assert_eq!(
            resolve_card_mode(NoteKind::Application, TopicState::Learning),
            MODE_NONE
        );
        assert_eq!(
            resolve_card_mode(NoteKind::Application, TopicState::Practicing),
            MODE_NONE
        );
        // Scaffolded at hierarchy, scaffold removed at mastering.
        assert_eq!(
            resolve_card_mode(NoteKind::Application, TopicState::Hierarchy),
            MODE_APPLICATION_SCAFFOLDED
        );
        assert_eq!(
            resolve_card_mode(NoteKind::Application, TopicState::Mastering),
            MODE_APPLICATION_UNSCAFFOLDED
        );
    }

    #[test]
    fn non_speedrun_card_is_always_none() {
        for state in [
            TopicState::Learning,
            TopicState::Practicing,
            TopicState::Hierarchy,
            TopicState::Mastering,
        ] {
            assert_eq!(resolve_card_mode(NoteKind::Other, state), MODE_NONE);
        }
    }

    // (e) Config round-trip.

    #[test]
    fn topic_state_round_trips_through_config() {
        let mut col = Collection::new();
        // Absent → learning (fail-safe default).
        assert_eq!(col.speedrun_topic_state(KINETICS), TopicState::Learning);

        col.set_speedrun_topic_state(KINETICS, TopicState::Hierarchy)
            .unwrap();
        assert_eq!(col.speedrun_topic_state(KINETICS), TopicState::Hierarchy);
        // An untouched topic is still the default; the map holds multiple keys.
        assert_eq!(col.speedrun_topic_state(STRUCTURE), TopicState::Learning);

        col.set_speedrun_topic_state(STRUCTURE, TopicState::Practicing)
            .unwrap();
        col.set_speedrun_topic_state(KINETICS, TopicState::Mastering)
            .unwrap();
        assert_eq!(col.speedrun_topic_state(KINETICS), TopicState::Mastering);
        assert_eq!(col.speedrun_topic_state(STRUCTURE), TopicState::Practicing);
    }

    // Integration: the transitions and mode resolution over real cards/revlog.

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

    /// Add a card of `nt`, optionally tagged with a taxonomy leaf id.
    fn add_card(col: &mut Collection, nt: &Notetype, tag: Option<&str>) -> CardId {
        let mut note = nt.new_note();
        note.set_field(0, "front").unwrap();
        if let Some(tag) = tag {
            note.tags.push(tag.to_string());
        }
        col.add_note(&mut note, DeckId(1)).unwrap();
        col.storage.card_ids_of_notes(&[note.id]).unwrap()[0]
    }

    /// Write `buttons` as graded review-kind revlog entries for `cid` (1 Again,
    /// 2 Hard, 3 Good, 4 Easy). `add_revlog_entry(.., true)` uniquifies ids so
    /// entries from different cards never collide.
    fn log_reviews(col: &mut Collection, cid: CardId, buttons: &[u8]) {
        for (i, button) in buttons.iter().enumerate() {
            let entry = RevlogEntry {
                id: RevlogId(TimestampSecs::now().0 * 1000 + i as i64),
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
    }

    #[test]
    fn record_answer_advances_when_signal_cleared() {
        let mut col = Collection::new();
        let nt = add_notetype_named(&mut col, "SpeedrunConcept");
        // 3 graded answers, all ≥Good: accuracy 1.0 ≥ 0.8 and 3 ≥ MIN_REPS.
        let cid = add_card(&mut col, &nt, Some(KINETICS));
        log_reviews(&mut col, cid, &[3, 3, 3]);

        assert_eq!(
            col.speedrun_record_answer(cid, Rating::Good).unwrap(),
            TopicState::Practicing
        );
        assert_eq!(col.speedrun_topic_state(KINETICS), TopicState::Practicing);
    }

    #[test]
    fn record_answer_holds_below_min_reps() {
        let mut col = Collection::new();
        let nt = add_notetype_named(&mut col, "SpeedrunConcept");
        // Only 1 graded answer (< MIN_REPS): no advance despite perfect accuracy.
        let cid = add_card(&mut col, &nt, Some(KINETICS));
        log_reviews(&mut col, cid, &[3]);

        assert_eq!(
            col.speedrun_record_answer(cid, Rating::Good).unwrap(),
            TopicState::Learning
        );
    }

    #[test]
    fn record_answer_holds_below_accuracy_threshold() {
        let mut col = Collection::new();
        let nt = add_notetype_named(&mut col, "SpeedrunConcept");
        // 4 reps but half are Again: accuracy 0.5 < 0.8 → no advance.
        let cid = add_card(&mut col, &nt, Some(KINETICS));
        log_reviews(&mut col, cid, &[3, 3, 1, 1]);

        assert_eq!(
            col.speedrun_record_answer(cid, Rating::Good).unwrap(),
            TopicState::Learning
        );
    }

    #[test]
    fn record_answer_again_demotes_topic() {
        let mut col = Collection::new();
        let nt = add_notetype_named(&mut col, "SpeedrunConcept");
        let cid = add_card(&mut col, &nt, Some(KINETICS));
        log_reviews(&mut col, cid, &[3, 3, 3]);
        col.set_speedrun_topic_state(KINETICS, TopicState::Hierarchy)
            .unwrap();

        // Again demotes regardless of the (cleared) signal.
        assert_eq!(
            col.speedrun_record_answer(cid, Rating::Again).unwrap(),
            TopicState::Practicing
        );
        assert_eq!(col.speedrun_topic_state(KINETICS), TopicState::Practicing);
    }

    #[test]
    fn record_answer_again_never_below_learning() {
        let mut col = Collection::new();
        let nt = add_notetype_named(&mut col, "SpeedrunConcept");
        let cid = add_card(&mut col, &nt, Some(KINETICS));
        log_reviews(&mut col, cid, &[1]);

        // Already learning by default; Again can't drop further.
        assert_eq!(
            col.speedrun_record_answer(cid, Rating::Again).unwrap(),
            TopicState::Learning
        );
    }

    #[test]
    fn record_answer_unmapped_card_is_noop() {
        let mut col = Collection::new();
        let nt = add_notetype_named(&mut col, "SpeedrunConcept");
        // No taxonomy tag → no topic → no transition.
        let cid = add_card(&mut col, &nt, None);
        log_reviews(&mut col, cid, &[3, 3, 3]);

        assert_eq!(
            col.speedrun_record_answer(cid, Rating::Good).unwrap(),
            TopicState::Learning
        );
    }

    #[test]
    fn advancement_signal_is_per_active_mode() {
        // A topic at practicing advances on its concept cards; an application
        // card's reviews must not be what clears the practicing→hierarchy gate.
        let mut col = Collection::new();
        let concept_nt = add_notetype_named(&mut col, "SpeedrunConcept");
        let app_nt = add_notetype_named(&mut col, "SpeedrunApplication");
        col.set_speedrun_topic_state(KINETICS, TopicState::Practicing)
            .unwrap();

        // Application card has a strong record, but the active mode at practicing
        // is concept, which has no graded history → no advance.
        let app = add_card(&mut col, &app_nt, Some(KINETICS));
        log_reviews(&mut col, app, &[3, 3, 3]);
        assert_eq!(
            col.speedrun_record_answer(app, Rating::Good).unwrap(),
            TopicState::Practicing,
            "application reviews don't clear the concept-mode gate"
        );

        // Give the concept card a clearing record → now it advances.
        let concept = add_card(&mut col, &concept_nt, Some(KINETICS));
        log_reviews(&mut col, concept, &[3, 3, 3]);
        assert_eq!(
            col.speedrun_record_answer(concept, Rating::Good).unwrap(),
            TopicState::Hierarchy
        );
    }

    #[test]
    fn card_mode_reflects_note_kind_and_topic_state() {
        let mut col = Collection::new();
        let concept_nt = add_notetype_named(&mut col, "SpeedrunConcept");
        let app_nt = add_notetype_named(&mut col, "SpeedrunApplication");
        let concept = add_card(&mut col, &concept_nt, Some(KINETICS));
        let app = add_card(&mut col, &app_nt, Some(KINETICS));

        // learning (default): concept learns; application is suppressed.
        assert_eq!(
            col.get_speedrun_card_mode(concept).unwrap(),
            MODE_CONCEPT_LEARN
        );
        assert_eq!(col.get_speedrun_card_mode(app).unwrap(), MODE_NONE);

        // hierarchy: concept practices; application is scaffolded.
        col.set_speedrun_topic_state(KINETICS, TopicState::Hierarchy)
            .unwrap();
        assert_eq!(
            col.get_speedrun_card_mode(concept).unwrap(),
            MODE_CONCEPT_PRACTICE
        );
        assert_eq!(
            col.get_speedrun_card_mode(app).unwrap(),
            MODE_APPLICATION_SCAFFOLDED
        );

        // mastering: the scaffold is removed.
        col.set_speedrun_topic_state(KINETICS, TopicState::Mastering)
            .unwrap();
        assert_eq!(
            col.get_speedrun_card_mode(app).unwrap(),
            MODE_APPLICATION_UNSCAFFOLDED
        );

        // A non-Speedrun card is always none.
        let basic_nt = col.basic_notetype();
        let basic = add_card(&mut col, &basic_nt, Some(KINETICS));
        assert_eq!(col.get_speedrun_card_mode(basic).unwrap(), MODE_NONE);
    }

    #[test]
    fn card_context_carries_mode_and_hierarchy_path() {
        let mut col = Collection::new();
        let concept_nt = add_notetype_named(&mut col, "SpeedrunConcept");
        let app_nt = add_notetype_named(&mut col, "SpeedrunApplication");
        let concept = add_card(&mut col, &concept_nt, Some(KINETICS));
        let app = add_card(&mut col, &app_nt, Some(KINETICS));

        // The breadcrumb path is the topic's taxonomy labels, regardless of
        // state; the mode tracks the topic state as usual.
        let (mode, path) = col.get_speedrun_card_context(concept).unwrap();
        assert_eq!(mode, MODE_CONCEPT_LEARN);
        assert_eq!(path, vec!["Biomolecules", "Enzymes", "Kinetics"]);

        // A suppressed application card (mode none) still reports its path; the
        // queue is what withholds it, not the context.
        let (mode, path) = col.get_speedrun_card_context(app).unwrap();
        assert_eq!(mode, MODE_NONE);
        assert_eq!(path, vec!["Biomolecules", "Enzymes", "Kinetics"]);

        // Advancing the topic changes only the mode, not the breadcrumb.
        col.set_speedrun_topic_state(KINETICS, TopicState::Hierarchy)
            .unwrap();
        let (mode, path) = col.get_speedrun_card_context(app).unwrap();
        assert_eq!(mode, MODE_APPLICATION_SCAFFOLDED);
        assert_eq!(path, vec!["Biomolecules", "Enzymes", "Kinetics"]);
    }

    #[test]
    fn card_context_is_empty_for_non_speedrun_and_unmapped_cards() {
        let mut col = Collection::new();
        let concept_nt = add_notetype_named(&mut col, "SpeedrunConcept");

        // A Speedrun card with no taxonomy tag: a real mode, but no breadcrumb.
        let untagged = add_card(&mut col, &concept_nt, None);
        let (mode, path) = col.get_speedrun_card_context(untagged).unwrap();
        assert_eq!(mode, MODE_CONCEPT_LEARN);
        assert!(path.is_empty(), "no topic -> no breadcrumb");

        // A non-Speedrun card is fully neutral even if it carries a taxonomy tag.
        let basic_nt = col.basic_notetype();
        let basic = add_card(&mut col, &basic_nt, Some(KINETICS));
        let (mode, path) = col.get_speedrun_card_context(basic).unwrap();
        assert_eq!(mode, MODE_NONE);
        assert!(path.is_empty());
    }

    #[test]
    fn progress_lists_covered_topics_with_states() {
        let mut col = Collection::new();
        let nt = add_notetype_named(&mut col, "SpeedrunConcept");
        add_card(&mut col, &nt, Some(KINETICS));
        add_card(&mut col, &nt, Some(STRUCTURE));
        add_card(&mut col, &nt, None); // unmapped: excluded from progress
        col.set_speedrun_topic_state(KINETICS, TopicState::Hierarchy)
            .unwrap();

        let progress = col.get_speedrun_progress(DeckId(1)).unwrap();
        // Only the two covered in-scope topics, sorted by id; KINETICS carries
        // its set state, STRUCTURE the default.
        assert_eq!(
            progress,
            vec![
                (STRUCTURE.to_string(), TopicState::Learning),
                (KINETICS.to_string(), TopicState::Hierarchy),
            ]
        );
    }
}
