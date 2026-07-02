# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Unit tests for the desktop card materialization (anki.speedrun.materialize).

The bespoke study screen's own materialize/reconcile runs in the shared Rust
engine (rslib/src/speedrun/study.rs) so desktop and AnkiDroid share it. This
Python module stays as the desktop seeder's helper (anki.speedrun.seed_deck),
so its contract is covered here: one card per authored concept, orphan removal,
FSRS enablement, and the ConceptId mapping. Needs rsbridge, so it runs under a
full build like the other test_speedrun_* suites.
"""

from __future__ import annotations

from typing import Any

# Import anki.collection before anki.decks so the anki package initializes in the
# right order (see test_speedrun_authoring / test_speedrun_progression).
import anki.collection  # noqa: F401
from anki.decks import DeckId
from anki.speedrun import authoring, materialize
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
