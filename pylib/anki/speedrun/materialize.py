# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Materialize authored Speedrun concepts into real Anki cards.

The bespoke study screen studies authored content (``speedrun_authoring``), but
the user wants **real FSRS** scheduling. FSRS only schedules real Anki cards, so
we mirror each authored concept as one card of a minimal ``SpeedrunItem`` note
type, keyed by ``ConceptId``. From then on the Anki engine owns *when* a concept
comes back (its interval), while the config store (``study.py``) owns *what*
interaction renders (the mastery state).

``reconcile`` is idempotent: it creates a card for every concept that lacks one,
removes cards whose concept no longer exists, and enables FSRS. It never imports
Qt, so it stays unit-testable against a plain collection; the mediasrv handler
runs it on study entry.
"""

from __future__ import annotations

from typing import Any

import anki.collection
from anki.decks import DeckId
from anki.models import NotetypeDict, NotetypeId
from anki.notes import NoteId
from anki.speedrun import authoring

ITEM_NOTETYPE_NAME = "SpeedrunItem"

# Field order is part of the contract. ConceptId keys the card back to the
# authored concept; Title is display-only (the screen reads content from the
# authored tree, not the note).
ITEM_FIELDS = ["ConceptId", "Title"]

# A minimal, always-non-empty front so the card always generates (Title may be
# blank). The card is never shown in the classic reviewer; the bespoke screen
# renders authored content instead.
_ITEM_QFMT = '<div class="speedrun-item">{{Title}}</div>'
_ITEM_AFMT = "{{FrontSide}}"


def _root(hierarchy: dict[str, Any]) -> dict[str, Any]:
    return hierarchy.get("root") or {}


def _walk_nodes(node: dict[str, Any]) -> list[dict[str, Any]]:
    nodes = [node]
    for child in node.get("children") or []:
        nodes.extend(_walk_nodes(child))
    return nodes


def _iter_concepts(hierarchy: dict[str, Any]) -> list[dict[str, Any]]:
    """Every authored concept in the tree, in document order."""
    concepts: list[dict[str, Any]] = []
    for node in _walk_nodes(_root(hierarchy)):
        concepts.extend(node.get("concepts") or [])
    return concepts


def item_notetype(col: anki.collection.Collection) -> NotetypeDict:
    """Build (without saving) the SpeedrunItem note type."""
    notetype = col.models.new(ITEM_NOTETYPE_NAME)
    for field_name in ITEM_FIELDS:
        col.models.add_field(notetype, col.models.new_field(field_name))
    template = col.models.new_template("Item")
    template["qfmt"] = _ITEM_QFMT
    template["afmt"] = _ITEM_AFMT
    col.models.add_template(notetype, template)
    return notetype


def install_item_notetype(col: anki.collection.Collection) -> NotetypeId:
    """Return the SpeedrunItem note type id, creating it if absent. Idempotent."""
    existing = col.models.id_for_name(ITEM_NOTETYPE_NAME)
    if existing is not None:
        return existing
    col.models.add_dict(item_notetype(col))
    new_id = col.models.id_for_name(ITEM_NOTETYPE_NAME)
    assert new_id is not None
    return new_id


def _ensure_fsrs(col: anki.collection.Collection) -> None:
    """Turn on FSRS so the engine schedules the materialized cards with genuine
    FSRS intervals. Collection-level toggle (Rust ``BoolKey::Fsrs``), idempotent."""
    if not col.get_config("fsrs", default=False):
        col.set_config("fsrs", True)


def reconcile(col: anki.collection.Collection, deck_id: str) -> dict[str, int]:
    """Ensure exactly one SpeedrunItem card per authored concept in the deck.

    Creates a card for every concept missing one (keyed by ``ConceptId``),
    removes cards whose concept id is no longer authored (or is a duplicate),
    and enables FSRS. Returns ``{created, removed, total}``. Idempotent: a second
    call with an unchanged hierarchy is a no-op that returns zero create/remove.
    """
    did = DeckId(int(deck_id))
    wanted: dict[str, str] = {}
    for concept in _iter_concepts(authoring.get_hierarchy(col, str(deck_id))):
        concept_id = str(concept.get("id") or "")
        if concept_id:
            wanted[concept_id] = str(concept.get("title") or "")

    notetype_id = install_item_notetype(col)
    notetype = col.models.get(notetype_id)
    assert notetype is not None

    kept: set[str] = set()
    orphans: list[NoteId] = []
    for card_id in col.decks.cids(did):
        note = col.get_note(col.get_card(card_id).nid)
        if note.mid != notetype_id:
            continue
        concept_id = note["ConceptId"]
        if concept_id in wanted and concept_id not in kept:
            kept.add(concept_id)
        else:
            orphans.append(note.id)

    created = 0
    for concept_id, title in wanted.items():
        if concept_id in kept:
            continue
        note = col.new_note(notetype)
        note["ConceptId"] = concept_id
        note["Title"] = title
        col.add_note(note, did)
        created += 1

    if orphans:
        col.remove_notes(orphans)

    _ensure_fsrs(col)

    return {"created": created, "removed": len(orphans), "total": len(wanted)}


def concept_id_for_note(col: anki.collection.Collection, note_id: NoteId) -> str:
    """The ConceptId a SpeedrunItem note maps to, or "" for any other note."""
    note = col.get_note(note_id)
    if note.mid != col.models.id_for_name(ITEM_NOTETYPE_NAME):
        return ""
    return note["ConceptId"]
