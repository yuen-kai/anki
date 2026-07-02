# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Unit tests for the native Speedrun authoring store.

These exercise the pure store logic (list/get/save/delete over collection
config) against a plain collection, with no Qt in the loop. Needs rsbridge, so
it runs under a full build like the other test_speedrun_* suites.
"""

from __future__ import annotations

# Import anki.collection before anki.decks so the anki package initializes in the
# right order; anki.decks first hits a circular import (see
# test_speedrun_progression).
import anki.collection  # noqa: F401
from anki.decks import DeckId
from anki.speedrun import authoring
from tests.shared import getEmptyCol

DEFAULT_DECK = DeckId(1)


def _hierarchy(deck_id: str, title: str) -> dict:
    return {
        "deckId": deck_id,
        "root": {"id": "root", "title": title, "children": [], "concepts": []},
    }


def test_list_decks_reports_todo_counts():
    col = getEmptyCol()
    try:
        physics = col.decks.id("Physics")
        assert physics is not None
        basic = col.models.by_name("Basic")
        assert basic is not None
        note = col.new_note(basic)
        note.fields[0] = "q"
        col.add_note(note, physics)

        rows = {row["name"]: row for row in authoring.list_decks(col)}
        assert "Physics" in rows
        assert rows["Physics"]["deckId"] == str(physics)
        # one fresh card -> one new -> To Do 1
        assert rows["Physics"]["todo"] == 1
    finally:
        col.close()


def test_save_new_hierarchy_creates_deck():
    col = getEmptyCol()
    try:
        result = authoring.save_hierarchy(
            col, _hierarchy(authoring.NEW_DECK_ID, "Biochem")
        )

        assert result["name"] == "Biochem"
        assert result["deckId"] not in ("", authoring.NEW_DECK_ID)
        did = DeckId(int(result["deckId"]))
        assert col.decks.name_if_exists(did) == "Biochem"

        # the blob is retrievable under the resolved deck id
        stored = authoring.get_hierarchy(col, result["deckId"])
        assert stored["deckId"] == result["deckId"]
        assert stored["root"]["title"] == "Biochem"
    finally:
        col.close()


def test_save_new_hierarchy_empty_title_is_noop():
    col = getEmptyCol()
    try:
        before = col.get_config(authoring.CONFIG_KEY, default=None)
        result = authoring.save_hierarchy(col, _hierarchy(authoring.NEW_DECK_ID, "   "))

        assert result == {"deckId": "", "name": ""}
        assert col.decks.id_for_name("   ") is None
        # nothing persisted
        assert col.get_config(authoring.CONFIG_KEY, default=None) == before
    finally:
        col.close()


def test_save_existing_hierarchy_renames_deck():
    col = getEmptyCol()
    try:
        created = authoring.save_hierarchy(
            col, _hierarchy(authoring.NEW_DECK_ID, "Old")
        )
        deck_id = created["deckId"]

        renamed = authoring.save_hierarchy(col, _hierarchy(deck_id, "New"))

        assert renamed["deckId"] == deck_id
        assert renamed["name"] == "New"
        assert col.decks.name_if_exists(DeckId(int(deck_id))) == "New"
        assert authoring.get_hierarchy(col, deck_id)["root"]["title"] == "New"
    finally:
        col.close()


def test_save_round_trips_nested_tree():
    col = getEmptyCol()
    try:
        blob = {
            "deckId": authoring.NEW_DECK_ID,
            "root": {
                "id": "root",
                "title": "Amino acids",
                "children": [
                    {
                        "id": "n1",
                        "title": "Structure",
                        "children": [],
                        "concepts": [
                            {
                                "id": "c1",
                                "title": "Zwitterions",
                                "content": "net-zero charge",
                                "problems": [
                                    {
                                        "id": "p1",
                                        "prompt": "At pH 7?",
                                        "choices": ["a", "b", "c", "d"],
                                        "correctIndex": 2,
                                    },
                                    {
                                        "id": "p2",
                                        "prompt": "Unset one",
                                        "choices": ["", "", "", ""],
                                        "correctIndex": -1,
                                    },
                                ],
                            }
                        ],
                    }
                ],
                "concepts": [],
            },
        }
        result = authoring.save_hierarchy(col, blob)

        reloaded = authoring.get_hierarchy(col, result["deckId"])
        concept = reloaded["root"]["children"][0]["concepts"][0]
        assert concept["title"] == "Zwitterions"
        assert concept["problems"][0]["correctIndex"] == 2
        assert concept["problems"][1]["correctIndex"] == -1
        assert concept["problems"][0]["choices"] == ["a", "b", "c", "d"]
    finally:
        col.close()


def test_get_hierarchy_defaults_to_deck_name():
    col = getEmptyCol()
    try:
        chem = col.decks.id("Chem")
        assert chem is not None

        default = authoring.get_hierarchy(col, str(chem))
        assert default["deckId"] == str(chem)
        assert default["root"]["title"] == "Chem"
        assert default["root"]["children"] == []
    finally:
        col.close()


def test_get_hierarchy_unknown_deck_is_empty():
    col = getEmptyCol()
    try:
        default = authoring.get_hierarchy(col, authoring.NEW_DECK_ID)
        assert default["root"]["title"] == ""
        assert default["root"]["children"] == []
    finally:
        col.close()


def test_delete_deck_removes_deck_and_blob():
    col = getEmptyCol()
    try:
        created = authoring.save_hierarchy(
            col, _hierarchy(authoring.NEW_DECK_ID, "Temp")
        )
        deck_id = created["deckId"]
        assert deck_id in col.get_config(authoring.CONFIG_KEY)

        authoring.delete_deck(col, deck_id)

        assert col.decks.name_if_exists(DeckId(int(deck_id))) is None
        assert deck_id not in col.get_config(authoring.CONFIG_KEY, default={})
    finally:
        col.close()
