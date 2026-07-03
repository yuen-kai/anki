# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Authored Speedrun deck for MCAT chapters 31-35 (organic chemistry & physics).

A second preloaded deck alongside :mod:`anki.speedrun.seed_deck`, built from real
Princeton Review MCAT content: five chapter leaves (two organic-chemistry, three
physics), each with four concepts of four problems (20 concepts, 80 problems).

The tree is large, so the data lives in its own sibling module
(:mod:`anki.speedrun.seed_mcat_ch31_35_data`, a single ``HIERARCHY`` dict) and
this module stays small: it loads that data and exposes the same pair the demo
seed does, ``build_hierarchy()`` and an idempotent :func:`seed`. Seeding goes
through the shared Rust authoring engine (``SpeedrunSaveHierarchy`` then
``SpeedrunNextCard`` to reconcile), so there is no second Python authoring path.
Ids are stable strings, so a reseed is a no-op.
"""

from __future__ import annotations

import copy
import json
from typing import Any

import anki.collection

from anki.speedrun.seed_mcat_ch31_35_data import HIERARCHY

DECK_NAME = "MCAT Ch 31–35: Organic Chemistry & Physics"

# The deckId the create flow sends before its Anki deck exists (mirrors the Rust
# authoring store's sentinel).
NEW_DECK_ID = "new"


def build_hierarchy(deck_id: str = NEW_DECK_ID) -> dict[str, Any]:
    """The full authored hierarchy as an authoring-store blob.

    ``deck_id`` defaults to the sentinel the create flow uses, so
    ``SpeedrunSaveHierarchy`` creates the backing deck from the root title on
    first save. The stored data is deep-copied so callers can't mutate it.
    """
    hierarchy = copy.deepcopy(HIERARCHY)
    hierarchy["deckId"] = deck_id
    return hierarchy


def seed(col: anki.collection.Collection) -> bool:
    """Create the MCAT deck if it is not already present. Idempotent.

    Returns True when it seeded, False when the deck already existed (so a
    repeat call, e.g. on every collection load, is a no-op that never clobbers
    the user's edits or study state). Goes through the shared Rust engine:
    ``SpeedrunSaveHierarchy`` creates the backing deck + stores the blob, then
    ``SpeedrunNextCard`` reconciles it (materializing one FSRS card per concept)
    so the deck shows a To Do count and is studyable right away.
    """
    if col.decks.id_for_name(DECK_NAME) is not None:
        return False

    saved = json.loads(
        col._backend.speedrun_save_hierarchy(
            json=json.dumps(build_hierarchy()).encode()
        )
    )
    deck_id = saved.get("deckId") or ""
    if not deck_id:
        return False

    # Reconcile (materialize the concept cards) via the study engine's next-card
    # entry point, which reconciles before returning.
    col._backend.speedrun_next_card(json=json.dumps({"deckId": deck_id}).encode())
    return True
