# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Unit tests for the bespoke Speedrun study backend.

Covers the Qt-free mastery-state store (anki.speedrun.study) — initial state,
the topic-gated learning flip, per-concept advance/demote and the practicing
floor — and the card materialization (anki.speedrun.materialize) — one card per
authored concept, orphan removal, ConceptId mapping, and FSRS enablement. Needs
rsbridge, so it runs under a full build like the other test_speedrun_* suites.
"""

from __future__ import annotations

from typing import cast

# Import anki.collection before anki.decks so the anki package initializes in the
# right order (see test_speedrun_authoring / test_speedrun_progression).
import anki.collection  # noqa: F401
from anki.decks import DeckId
from anki.scheduler.v3 import CardAnswer
from anki.scheduler.v3 import Scheduler as V3Scheduler
from anki.speedrun import authoring, materialize, study
from tests.shared import getEmptyCol


def _concept(concept_id: str) -> dict:
    return {
        "id": concept_id,
        "title": concept_id.upper(),
        "content": f"about {concept_id}",
        "problems": [],
    }


def _blob(deck_id: str, leaves: dict[str, list[str]]) -> dict:
    """A hierarchy blob whose root has one child node per leaf title, each
    holding the given concept ids."""
    return {
        "deckId": deck_id,
        "root": {
            "id": "root",
            "title": "Biochem",
            "concepts": [],
            "children": [
                {
                    "id": f"node-{title}",
                    "title": title,
                    "children": [],
                    "concepts": [_concept(cid) for cid in cids],
                }
                for title, cids in leaves.items()
            ],
        },
    }


def _store_hierarchy(col, deck_id: str, leaves: dict[str, list[str]]) -> None:
    col.set_config(authoring.CONFIG_KEY, {str(deck_id): _blob(str(deck_id), leaves)})


# --- Store: initial state ---------------------------------------------------


def test_absent_concept_defaults_to_learning_unseen():
    col = getEmptyCol()
    try:
        _store_hierarchy(col, "1", {"Amino acids": ["c1", "c2"]})
        state = study.get_study_state(col, "1")["progress"]
        assert state == {
            "c1": {"state": "learning", "seen": False},
            "c2": {"state": "learning", "seen": False},
        }
    finally:
        col.close()


# --- Store: topic-gated learning flip (ST7) ---------------------------------


def test_record_learned_is_topic_gated():
    col = getEmptyCol()
    try:
        _store_hierarchy(col, "1", {"Amino acids": ["c1", "c2"]})

        # Learning one of two concepts marks it seen but does not flip the topic.
        first = study.record_learned(col, "1", ["c1"])
        assert first["upgraded"] is False
        assert first["conceptIds"] == []
        progress = study.get_study_state(col, "1")["progress"]
        assert progress["c1"] == {"state": "learning", "seen": True}
        assert progress["c2"] == {"state": "learning", "seen": False}

        # Learning the last concept flips the whole leaf together.
        second = study.record_learned(col, "1", ["c2"])
        assert second["upgraded"] is True
        assert second["from"] == "learning"
        assert second["to"] == "practicing"
        assert set(second["conceptIds"]) == {"c1", "c2"}
        progress = study.get_study_state(col, "1")["progress"]
        assert progress["c1"] == {"state": "practicing", "seen": True}
        assert progress["c2"] == {"state": "practicing", "seen": True}
    finally:
        col.close()


def test_record_learned_only_flips_the_completed_leaf():
    col = getEmptyCol()
    try:
        _store_hierarchy(col, "1", {"Structure": ["a1"], "Kinetics": ["b1", "b2"]})

        # Completing the single-concept leaf flips only it; the other leaf waits.
        result = study.record_learned(col, "1", ["a1"])
        assert result["upgraded"] is True
        assert result["conceptIds"] == ["a1"]
        progress = study.get_study_state(col, "1")["progress"]
        assert progress["a1"]["state"] == "practicing"
        assert progress["b1"]["state"] == "learning"
        assert progress["b2"]["state"] == "learning"
    finally:
        col.close()


def test_next_learning_block_reports_the_first_unfinished_topic():
    col = getEmptyCol()
    try:
        _store_hierarchy(col, "1", {"Structure": ["a1"], "Kinetics": ["b1", "b2"]})
        study.record_learned(col, "1", ["a1"])  # finishes Structure

        hierarchy = authoring.get_hierarchy(col, "1")
        progress = study.get_study_state(col, "1")["progress"]
        block = study.next_learning_block(hierarchy, progress)
        assert block is not None
        assert block["topicNodeId"] == "node-Kinetics"
        assert block["conceptIds"] == ["b1", "b2"]
        assert block["learnedCount"] == 0
        assert block["totalCount"] == 2

        # Once every concept is learned there is no block left.
        study.record_learned(col, "1", ["b1", "b2"])
        progress = study.get_study_state(col, "1")["progress"]
        assert study.next_learning_block(hierarchy, progress) is None
    finally:
        col.close()


# --- Store: per-concept advance / demote (ST8) ------------------------------


def _practice(col, deck_id: str, concept_ids: list[str]) -> None:
    """Flip a leaf's concepts to practicing by learning them all."""
    study.record_learned(col, deck_id, concept_ids)


def test_record_answer_advances_once_signal_clears():
    col = getEmptyCol()
    try:
        _store_hierarchy(col, "1", {"Amino acids": ["c1", "c2"]})
        _practice(col, "1", ["c1", "c2"])

        # One Good is below MIN_REPS, so the concept stays practicing.
        first = study.record_answer(col, "1", "c1", study.RATING_GOOD)
        assert first == {
            "state": "practicing",
            "upgraded": False,
            "from": "practicing",
            "to": "practicing",
        }

        # A second Good clears the signal and advances one state (Applying).
        second = study.record_answer(col, "1", "c1", study.RATING_GOOD)
        assert second["upgraded"] is True
        assert second["from"] == "practicing"
        assert second["to"] == "hierarchy"
        assert second["state"] == "hierarchy"
    finally:
        col.close()


def test_record_answer_demotes_on_again():
    col = getEmptyCol()
    try:
        _store_hierarchy(col, "1", {"Amino acids": ["c1", "c2"]})
        _practice(col, "1", ["c1", "c2"])
        # Advance c1 to hierarchy (Applying).
        study.record_answer(col, "1", "c1", study.RATING_GOOD)
        study.record_answer(col, "1", "c1", study.RATING_GOOD)
        assert study.get_study_state(col, "1")["progress"]["c1"]["state"] == "hierarchy"

        # Again demotes exactly one state.
        result = study.record_answer(col, "1", "c1", study.RATING_AGAIN)
        assert result["from"] == "hierarchy"
        assert result["to"] == "practicing"
        assert result["upgraded"] is False
        assert result["state"] == "practicing"
    finally:
        col.close()


def test_record_answer_floors_at_practicing():
    col = getEmptyCol()
    try:
        _store_hierarchy(col, "1", {"Amino acids": ["c1", "c2"]})
        _practice(col, "1", ["c1", "c2"])

        # Again on a practicing concept never drops below practicing (never back
        # into the learning topic block).
        result = study.record_answer(col, "1", "c1", study.RATING_AGAIN)
        assert result["state"] == "practicing"
        assert result["to"] == "practicing"
        assert (
            study.get_study_state(col, "1")["progress"]["c1"]["state"] == "practicing"
        )
    finally:
        col.close()


# --- Materialize / reconcile (ST12) -----------------------------------------


def _item_cards(col, did: DeckId) -> dict[str, int]:
    """conceptId -> cardId for every SpeedrunItem card in the deck."""
    item_id = col.models.id_for_name(materialize.ITEM_NOTETYPE_NAME)
    out: dict[str, int] = {}
    for card_id in col.decks.cids(did):
        note = col.get_note(col.get_card(card_id).nid)
        if note.mid == item_id:
            out[note["ConceptId"]] = card_id
    return out


def test_reconcile_creates_one_card_per_concept_and_enables_fsrs():
    col = getEmptyCol()
    try:
        created = authoring.save_hierarchy(
            col, _blob(authoring.NEW_DECK_ID, {"Amino acids": ["c1", "c2"]})
        )
        deck_id = created["deckId"]
        did = DeckId(int(deck_id))

        result = materialize.reconcile(col, deck_id)
        assert result == {"created": 2, "removed": 0, "total": 2}

        cards = _item_cards(col, did)
        assert set(cards) == {"c1", "c2"}, "one card per concept, keyed by ConceptId"
        assert col.get_config("fsrs", default=False) is True, "FSRS enabled"

        # Idempotent: a second reconcile over the same tree changes nothing.
        assert materialize.reconcile(col, deck_id) == {
            "created": 0,
            "removed": 0,
            "total": 2,
        }
        assert set(_item_cards(col, did)) == {"c1", "c2"}
    finally:
        col.close()


def test_reconcile_removes_orphaned_cards():
    col = getEmptyCol()
    try:
        created = authoring.save_hierarchy(
            col, _blob(authoring.NEW_DECK_ID, {"Amino acids": ["c1", "c2"]})
        )
        deck_id = created["deckId"]
        did = DeckId(int(deck_id))
        materialize.reconcile(col, deck_id)
        assert set(_item_cards(col, did)) == {"c1", "c2"}

        # Drop c2 from the authored tree; reconcile must remove its card.
        authoring.save_hierarchy(col, _blob(deck_id, {"Amino acids": ["c1"]}))
        result = materialize.reconcile(col, deck_id)
        assert result == {"created": 0, "removed": 1, "total": 1}
        assert set(_item_cards(col, did)) == {"c1"}
    finally:
        col.close()


def test_concept_id_for_note_maps_card_to_concept():
    col = getEmptyCol()
    try:
        created = authoring.save_hierarchy(
            col, _blob(authoring.NEW_DECK_ID, {"Amino acids": ["c1"]})
        )
        deck_id = created["deckId"]
        did = DeckId(int(deck_id))
        materialize.reconcile(col, deck_id)

        card_id = _item_cards(col, did)["c1"]
        note_id = col.get_card(card_id).nid
        assert materialize.concept_id_for_note(col, note_id) == "c1"

        # A non-SpeedrunItem note maps to no concept.
        basic = col.models.by_name("Basic")
        assert basic is not None
        note = col.new_note(basic)
        note.fields[0] = "front"
        col.add_note(note, did)
        assert materialize.concept_id_for_note(col, note.id) == ""
    finally:
        col.close()


def test_materialized_card_grades_through_fsrs():
    """The mediasrv answer path (build_answer + answer_card, from_queue=False)
    grades a materialized card through real FSRS: it writes a revlog entry and
    moves the card out of the `new` state with an FSRS memory state (ST12)."""
    col = getEmptyCol()
    try:
        created = authoring.save_hierarchy(
            col, _blob(authoring.NEW_DECK_ID, {"Amino acids": ["c1"]})
        )
        deck_id = created["deckId"]
        did = DeckId(int(deck_id))
        materialize.reconcile(col, deck_id)
        col.decks.set_current(did)

        sched = cast(V3Scheduler, col.sched)
        card = col.get_card(_item_cards(col, did)["c1"])
        card.start_timer()
        states = col._backend.get_scheduling_states(card.id)
        answer = sched.build_answer(
            card=card, states=states, rating=CardAnswer.GOOD, from_queue=False
        )
        revlog_before = col.db.scalar("select count() from revlog")
        sched.answer_card(answer)

        assert col.db.scalar("select count() from revlog") == revlog_before + 1
        graded = col.get_card(card.id)
        assert graded.type != 0, "no longer a New card"
        assert graded.memory_state is not None, "FSRS memory state assigned"
    finally:
        col.close()
