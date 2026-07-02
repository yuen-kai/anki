# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Native Speedrun authoring store (the native-store plan choice).

One JSON blob per deck lives under the ``speedrun_authoring`` collection-config
map, keyed by the Anki deck id. A deck row is a real Anki deck (so Study and the
To Do count keep working); its authored tree/concepts/problems attach by deck id.
Nested nodes are store-only, they are not Anki subdecks.

Shapes (mirrored by the frontend's ``lib.ts``)::

    Hierarchy { deckId, root: Node }
    Node      { id, title, children: Node[], concepts: Concept[] }
    Concept   { id, title, content, problems: Problem[] }
    Problem   { id, prompt, choices: [c0, c1, c2, c3], correctIndex }  # -1 = unset

This module never imports Qt, so the store stays unit-testable against a plain
collection; the mediasrv handlers own the JSON <-> protobuf <-> HTTP plumbing.
"""

from __future__ import annotations

from typing import Any

import anki.collection
from anki.decks import DeckId

CONFIG_KEY = "speedrun_authoring"

# The deckId the create flow sends before its Anki deck exists.
NEW_DECK_ID = "new"


def _store(col: anki.collection.Collection) -> dict[str, Any]:
    return col.get_config(CONFIG_KEY, default={}) or {}


def _empty_hierarchy(deck_id: str, title: str) -> dict[str, Any]:
    return {
        "deckId": deck_id,
        "root": {"id": "root", "title": title, "children": [], "concepts": []},
    }


def list_decks(col: anki.collection.Collection) -> list[dict[str, Any]]:
    """Every top-level Anki deck as a row: deckId, name, and its To Do count
    (new + learn + review from the due tree). This replaces the deck browser."""
    tree = col.sched.deck_due_tree()
    return [
        {
            "deckId": str(child.deck_id),
            "name": child.name,
            "todo": child.new_count + child.learn_count + child.review_count,
        }
        for child in tree.children
    ]


def get_hierarchy(col: anki.collection.Collection, deck_id: str) -> dict[str, Any]:
    """The stored blob for a deck, or a fresh one seeded with the deck's current
    name so the editor always has a root title to show."""
    stored = _store(col).get(str(deck_id))
    if stored is not None:
        return stored
    title = ""
    if deck_id and deck_id != NEW_DECK_ID:
        title = col.decks.name_if_exists(DeckId(int(deck_id))) or ""
    return _empty_hierarchy(str(deck_id), title)


def save_hierarchy(
    col: anki.collection.Collection, hierarchy: dict[str, Any]
) -> dict[str, Any]:
    """Persist a deck's authored tree and return the resolved ``{deckId, name}``.

    Creates the backing Anki deck for a new hierarchy (root title = deck name) or
    renames it when the root title changed, then writes the blob under the deck
    id. A new hierarchy with an empty root title is a no-op: there is nothing to
    key the blob on yet, so it returns empty ids."""
    deck_id = str(hierarchy.get("deckId") or "")
    root = hierarchy.get("root") or {}
    title = (root.get("title") or "").strip()

    if not deck_id or deck_id == NEW_DECK_ID:
        if not title:
            return {"deckId": "", "name": ""}
        did = col.decks.id(title)
        assert did is not None
    else:
        did = DeckId(int(deck_id))
        current = col.decks.name_if_exists(did)
        if title and current and current != title:
            col.decks.rename(did, title)

    hierarchy["deckId"] = str(did)
    store = _store(col)
    store[str(did)] = hierarchy
    col.set_config(CONFIG_KEY, store)

    return {"deckId": str(did), "name": col.decks.name_if_exists(did) or title}


def delete_deck(col: anki.collection.Collection, deck_id: str) -> None:
    """Remove the backing Anki deck and drop its authored blob."""
    did = DeckId(int(deck_id))
    col.decks.remove([did])
    store = _store(col)
    if str(did) in store:
        del store[str(did)]
        col.set_config(CONFIG_KEY, store)
