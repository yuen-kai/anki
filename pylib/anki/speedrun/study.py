# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Per-concept mastery state for the bespoke Speedrun study screen.

FSRS owns *timing* (when a card is due), via ``answer_card`` on the materialized
cards. This module owns *mode* (what interaction renders), a small map in the
collection config under ``speedrun_study_progress`` — the same native-store
pattern as ``authoring.py``, no Rust. The two are deliberately separate.

Shape (per deck, keyed by the Anki deck id)::

    speedrun_study_progress = {
        "<deckId>": {
            "<conceptId>": { "state": str, "seen": bool, "ratings": [int, ...] }
        }
    }

States mirror the taxonomy lifecycle (``progression.rs``); the internal
``hierarchy`` state displays as "Applying" in the UI::

    learning -> practicing -> hierarchy(Applying) -> mastering

- **Learning is topic-gated**: a concept is ``learning`` until every concept in
  its authored leaf node has been seen, at which point they all flip to
  ``practicing`` together (``record_learned``).
- **Later stages are per-concept**: an answer advances one state once the
  concept's recent ratings clear the mastery signal, or demotes one state on
  ``Again``. Demotion floors at ``practicing`` so a lapse never re-enters the
  topic block (``record_answer``).

Qt-free, so it stays unit-testable against a plain collection; the mediasrv
handlers own the JSON <-> HTTP plumbing.
"""

from __future__ import annotations

from typing import Any

import anki.collection
from anki.speedrun import authoring

CONFIG_KEY = "speedrun_study_progress"

STATE_LEARNING = "learning"
STATE_PRACTICING = "practicing"
STATE_HIERARCHY = "hierarchy"
STATE_MASTERING = "mastering"

# The mastery ladder, low to high. Advancement walks up one rung; demotion walks
# down one but never below index 1 (practicing).
_LADDER = [STATE_LEARNING, STATE_PRACTICING, STATE_HIERARCHY, STATE_MASTERING]
_PRACTICING_INDEX = _LADDER.index(STATE_PRACTICING)

# Mastery-signal tunables (mirror rslib/src/speedrun/progression.rs).
ACC_THRESHOLD = 0.8
MIN_REPS = 2
MASTERY_REVIEW_WINDOW = 50

# Difficulty rating encoding as sent by the screen: 1..4 = Again/Hard/Good/Easy.
RATING_AGAIN = 1
RATING_GOOD = 3


# --- Hierarchy helpers (pure) ----------------------------------------------


def _root(hierarchy: dict[str, Any]) -> dict[str, Any]:
    return hierarchy.get("root") or {}


def _walk_nodes(node: dict[str, Any]) -> list[dict[str, Any]]:
    nodes = [node]
    for child in node.get("children") or []:
        nodes.extend(_walk_nodes(child))
    return nodes


def _concept_leaves(hierarchy: dict[str, Any]) -> list[dict[str, Any]]:
    """Every node that directly holds concepts (a topic block), in tree order."""
    return [node for node in _walk_nodes(_root(hierarchy)) if node.get("concepts")]


def _concept_ids(node: dict[str, Any]) -> list[str]:
    return [str(c.get("id") or "") for c in (node.get("concepts") or []) if c.get("id")]


# --- Config read/write -----------------------------------------------------


def _all(col: anki.collection.Collection) -> dict[str, Any]:
    return col.get_config(CONFIG_KEY, default={}) or {}


def _deck_progress(col: anki.collection.Collection, deck_id: str) -> dict[str, Any]:
    return dict(_all(col).get(str(deck_id), {}))


def _save(
    col: anki.collection.Collection, deck_id: str, progress: dict[str, Any]
) -> None:
    store = _all(col)
    store[str(deck_id)] = progress
    col.set_config(CONFIG_KEY, store)


def _entry(progress: dict[str, Any], concept_id: str) -> dict[str, Any]:
    """The stored entry for a concept, normalized; absent = fresh learning."""
    raw = progress.get(concept_id) or {}
    return {
        "state": raw.get("state", STATE_LEARNING),
        "seen": bool(raw.get("seen", False)),
        "ratings": [int(r) for r in (raw.get("ratings") or [])],
    }


# --- Ladder transitions (pure) ---------------------------------------------


def _advanced(state: str) -> str:
    index = _LADDER.index(state)
    return _LADDER[min(index + 1, len(_LADDER) - 1)]


def _demoted(state: str) -> str:
    index = _LADDER.index(state)
    return _LADDER[max(index - 1, _PRACTICING_INDEX)]


def _signal_cleared(ratings: list[int]) -> bool:
    """True when recent ratings clear the advancement signal: at least MIN_REPS
    of the last MASTERY_REVIEW_WINDOW answers, with a >=Good rate >= ACC_THRESHOLD."""
    window = ratings[-MASTERY_REVIEW_WINDOW:]
    if len(window) < MIN_REPS:
        return False
    good = sum(1 for r in window if r >= RATING_GOOD)
    return good / len(window) >= ACC_THRESHOLD


# --- Public API ------------------------------------------------------------


def get_study_state(col: anki.collection.Collection, deck_id: str) -> dict[str, Any]:
    """The mastery state of every authored concept in the deck: ``{progress:
    {conceptId: {state, seen}}}``. An absent concept defaults to learning/unseen,
    so the screen always gets a complete map."""
    progress = _deck_progress(col, deck_id)
    hierarchy = authoring.get_hierarchy(col, str(deck_id))
    out: dict[str, Any] = {}
    for node in _concept_leaves(hierarchy):
        for concept_id in _concept_ids(node):
            entry = _entry(progress, concept_id)
            out[concept_id] = {"state": entry["state"], "seen": entry["seen"]}
    return {"progress": out}


def next_learning_block(
    hierarchy: dict[str, Any], progress: dict[str, Any]
) -> dict[str, Any] | None:
    """The first leaf topic that still has a concept in ``learning``, as the
    block to teach, or None when every concept has been learned. ``progress`` is
    the ``{conceptId: {state, seen}}`` map from :func:`get_study_state`."""
    for node in _concept_leaves(hierarchy):
        ids = _concept_ids(node)
        states = {cid: (progress.get(cid) or {}) for cid in ids}
        if any(
            states[cid].get("state", STATE_LEARNING) == STATE_LEARNING for cid in ids
        ):
            learned = sum(1 for cid in ids if states[cid].get("seen", False))
            return {
                "topicNodeId": str(node.get("id") or ""),
                "conceptIds": ids,
                "learnedCount": learned,
                "totalCount": len(ids),
            }
    return None


def record_learned(
    col: anki.collection.Collection, deck_id: str, concept_ids: list[str]
) -> dict[str, Any]:
    """Mark concepts seen; when every concept in a touched leaf topic is seen,
    flip that whole topic ``learning -> practicing`` together (topic-gated, ST7).

    Returns ``{upgraded, from, to, conceptIds}`` — the concepts flipped this call
    (empty when the topic isn't fully learned yet)."""
    hierarchy = authoring.get_hierarchy(col, str(deck_id))
    progress = _deck_progress(col, deck_id)

    touched = {str(cid) for cid in concept_ids}
    for concept_id in touched:
        entry = _entry(progress, concept_id)
        entry["seen"] = True
        progress[concept_id] = entry

    upgraded: list[str] = []
    for node in _concept_leaves(hierarchy):
        ids = _concept_ids(node)
        if not touched.intersection(ids):
            continue
        if not all(_entry(progress, cid)["seen"] for cid in ids):
            continue
        for concept_id in ids:
            entry = _entry(progress, concept_id)
            if entry["state"] == STATE_LEARNING:
                entry["state"] = STATE_PRACTICING
                progress[concept_id] = entry
                upgraded.append(concept_id)

    _save(col, deck_id, progress)

    if upgraded:
        return {
            "upgraded": True,
            "from": STATE_LEARNING,
            "to": STATE_PRACTICING,
            "conceptIds": upgraded,
        }
    return {
        "upgraded": False,
        "from": STATE_LEARNING,
        "to": STATE_LEARNING,
        "conceptIds": [],
    }


def record_answer(
    col: anki.collection.Collection, deck_id: str, concept_id: str, rating: int
) -> dict[str, Any]:
    """Record a difficulty rating against a concept and move its mastery state
    (ST8). ``Again`` demotes one state (floor ``practicing``); any other rating
    advances one state once the rolling signal clears. Mastery state only — FSRS
    timing is handled separately by ``answer_card``.

    Returns ``{state, upgraded, from, to}``."""
    rating = int(rating)
    progress = _deck_progress(col, deck_id)
    entry = _entry(progress, concept_id)

    # Answers only reach concepts at practicing or above; floor the effective
    # pre-answer state so a stray learning concept never drops below practicing.
    current = entry["state"]
    if _LADDER.index(current) < _PRACTICING_INDEX:
        current = STATE_PRACTICING

    entry["ratings"] = (entry["ratings"] + [rating])[-MASTERY_REVIEW_WINDOW:]
    entry["seen"] = True

    if rating == RATING_AGAIN:
        new_state = _demoted(current)
    elif _signal_cleared(entry["ratings"]):
        new_state = _advanced(current)
    else:
        new_state = current

    entry["state"] = new_state
    progress[concept_id] = entry
    _save(col, deck_id, progress)

    return {
        "state": new_state,
        "upgraded": _LADDER.index(new_state) > _LADDER.index(current),
        "from": current,
        "to": new_state,
    }
