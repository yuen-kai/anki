# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Unit tests for the authored MCAT deck (anki.speedrun.seed_mcat_ch31_35).

Checks the fixed shape the study/review screens rely on (5 chapter leaves, each
with exactly 4 concepts of exactly 4 well-formed 4-choice problems, i.e. 20
concepts / 80 problems) and the idempotent :func:`seed` (creates the deck +
materializes one card per concept once, no-op thereafter). Needs rsbridge, so it
runs under a full build like the other test_speedrun_* suites.
"""

from __future__ import annotations

import json
from collections.abc import Iterator
from typing import Any

# anki.collection is imported before the other anki submodules so the package
# initializes in the right order (see the sibling test_speedrun_* suites).
import anki.collection  # noqa: F401
from anki.speedrun import seed_mcat_ch31_35 as seed_mcat
from tests.shared import getEmptyCol


def _walk(node: dict[str, Any]) -> Iterator[dict[str, Any]]:
    yield node
    for child in node.get("children", []):
        yield from _walk(child)


def _leaves(hierarchy: dict[str, Any]) -> list[dict[str, Any]]:
    return [n for n in _walk(hierarchy["root"]) if n.get("concepts")]


def _all_concepts(hierarchy: dict[str, Any]) -> list[dict[str, Any]]:
    concepts: list[dict[str, Any]] = []
    for node in _walk(hierarchy["root"]):
        concepts.extend(node.get("concepts", []))
    return concepts


def test_hierarchy_shape_is_studyable():
    hierarchy = seed_mcat.build_hierarchy()

    assert hierarchy["deckId"] == seed_mcat.NEW_DECK_ID
    root = hierarchy["root"]
    assert root["title"] == seed_mcat.DECK_NAME
    assert not root["concepts"], "root is structural, not a topic leaf"

    ids: set[str] = set()

    def visit(node: dict[str, Any], depth: int) -> None:
        node_id = node.get("id") or ""
        assert node_id and node_id not in ids, f"node id must be unique: {node_id!r}"
        ids.add(node_id)
        for child in node.get("children", []):
            visit(child, depth + 1)
        if node.get("concepts"):
            assert not node["children"], f"{node_id} holds concepts, so it is a leaf"
            assert depth >= 2, f"{node_id} concepts must sit at depth >= 2"

    visit(root, 0)

    leaves = _leaves(hierarchy)
    assert len(leaves) == 5, "two organic-chemistry + three physics chapter leaves"
    for leaf in leaves:
        assert len(leaf["concepts"]) == 4, f"{leaf['title']} must have 4 concepts"

    concepts = _all_concepts(hierarchy)
    assert len(concepts) == 20

    problem_count = 0
    for concept in concepts:
        cid = concept.get("id") or ""
        assert cid and cid not in ids, f"concept id must be unique: {cid!r}"
        ids.add(cid)
        assert concept.get("title", "").strip(), f"{cid} needs a title"
        assert concept.get("content", "").strip(), f"{cid} needs content"
        problems = concept.get("problems", [])
        assert len(problems) == 4, f"{cid} must have exactly four problems"
        for problem in problems:
            problem_count += 1
            pid = problem.get("id") or ""
            assert pid and pid not in ids, f"problem id must be unique: {pid!r}"
            ids.add(pid)
            assert problem.get("prompt", "").strip(), f"{pid} needs a prompt"
            choices = problem.get("choices", [])
            assert len(choices) == 4, f"{pid} must have exactly four choices"
            assert all(c.strip() for c in choices), f"{pid} has a blank choice"
            correct = problem.get("correctIndex")
            assert isinstance(correct, int) and 0 <= correct < 4, (
                f"{pid} needs a valid correctIndex"
            )

    assert problem_count == 80


def test_seed_creates_deck_and_materializes_once():
    col = getEmptyCol()
    concept_count = len(_all_concepts(seed_mcat.build_hierarchy()))

    assert col.decks.id_for_name(seed_mcat.DECK_NAME) is None
    assert seed_mcat.seed(col) is True

    did = col.decks.id_for_name(seed_mcat.DECK_NAME)
    assert did is not None

    # The authored blob is stored under the real deck id (read back through the
    # shared Rust engine), and every concept was materialized into one card.
    stored = json.loads(
        col._backend.speedrun_study_hierarchy(
            json=json.dumps({"deckId": str(did)}).encode()
        )
    )
    assert len(_all_concepts(stored)) == concept_count
    assert len(col.decks.cids(did)) == concept_count, "one card per concept"


def test_seed_is_idempotent():
    col = getEmptyCol()

    assert seed_mcat.seed(col) is True
    did = col.decks.id_for_name(seed_mcat.DECK_NAME)
    assert did is not None
    card_count = len(col.decks.cids(did))

    # A second call is a no-op: no reseed, no duplicate deck, no extra cards.
    assert seed_mcat.seed(col) is False
    assert col.decks.id_for_name(seed_mcat.DECK_NAME) == did
    assert len(col.decks.cids(did)) == card_count
