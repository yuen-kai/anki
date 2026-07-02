# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Unit tests for the complete authored demo deck (anki.speedrun.seed_deck).

Covers the pure hierarchy shape the study/review screens rely on (depth, a
multi-concept leaf, and well-formed 4-choice problems) and the idempotent
:func:`seed` (creates the deck + materializes cards once, no-op thereafter).
Needs rsbridge, so it runs under a full build like the other test_speedrun_*
suites.
"""

from __future__ import annotations

from collections.abc import Iterator
from typing import Any

# anki.collection is imported before the other anki submodules so the package
# initializes in the right order (see the sibling test_speedrun_* suites).
import anki.collection  # noqa: F401
from anki.speedrun import authoring, materialize, seed_deck
from tests.shared import getEmptyCol


def _walk(node: dict[str, Any]) -> Iterator[dict[str, Any]]:
    yield node
    for child in node.get("children", []):
        yield from _walk(child)


def _all_concepts(hierarchy: dict[str, Any]) -> list[dict[str, Any]]:
    concepts: list[dict[str, Any]] = []
    for node in _walk(hierarchy["root"]):
        concepts.extend(node.get("concepts", []))
    return concepts


def test_hierarchy_shape_is_studyable():
    hierarchy = seed_deck.build_hierarchy()

    assert hierarchy["deckId"] == authoring.NEW_DECK_ID
    root = hierarchy["root"]
    assert root["title"] == seed_deck.DEMO_DECK_NAME
    assert not root["concepts"], "root is structural, not a topic leaf"
    assert len(root["children"]) >= 2

    ids: set[str] = set()
    multi_concept_leaves = 0

    def visit(node: dict[str, Any], depth: int) -> None:
        nonlocal multi_concept_leaves
        node_id = node.get("id") or ""
        assert node_id and node_id not in ids, f"node id must be unique: {node_id!r}"
        ids.add(node_id)

        concepts = node.get("concepts", [])
        children = node.get("children", [])
        if concepts:
            # A concept-holding node is a leaf, and sits deep enough (root ->
            # branch -> leaf) that the Applying scaffold has levels to pick.
            assert not children, f"{node_id} holds concepts, so it must be a leaf"
            assert depth >= 2, f"{node_id} concepts must sit at depth >= 2"
            if len(concepts) >= 2:
                multi_concept_leaves += 1
        for child in children:
            visit(child, depth + 1)

    visit(root, 0)

    # At least one leaf with two concepts exercises the topic-gated learning
    # block (both must be seen before the topic flips to practicing).
    assert multi_concept_leaves >= 1

    concepts = _all_concepts(hierarchy)
    assert len(concepts) >= 8, "a complete deck spans the biomolecules topics"

    for concept in concepts:
        cid = concept.get("id") or ""
        assert cid and cid not in ids, f"concept id must be unique: {cid!r}"
        ids.add(cid)
        assert concept.get("title", "").strip(), f"{cid} needs a title"
        assert concept.get("content", "").strip(), f"{cid} needs content"
        problems = concept.get("problems", [])
        # The Learn stage shows the first two problems as contrasting cases.
        assert len(problems) >= 2, f"{cid} needs at least two problems"
        for problem in problems:
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


def test_seed_creates_deck_and_materializes_once():
    col = getEmptyCol()
    concept_count = len(_all_concepts(seed_deck.build_hierarchy()))

    assert col.decks.id_for_name(seed_deck.DEMO_DECK_NAME) is None
    assert seed_deck.seed(col) is True

    did = col.decks.id_for_name(seed_deck.DEMO_DECK_NAME)
    assert did is not None

    # The authored blob is stored under the real deck id, and every concept was
    # materialized into exactly one card.
    stored = authoring.get_hierarchy(col, str(did))
    assert len(_all_concepts(stored)) == concept_count
    reconciled = materialize.reconcile(col, str(did))
    assert reconciled["total"] == concept_count
    assert reconciled["created"] == 0, "seed already materialized every concept"


def test_seed_is_idempotent():
    col = getEmptyCol()

    assert seed_deck.seed(col) is True
    did = col.decks.id_for_name(seed_deck.DEMO_DECK_NAME)
    assert did is not None
    card_count = len(col.decks.cids(did))

    # A second call is a no-op: no reseed, no duplicate deck, no extra cards.
    assert seed_deck.seed(col) is False
    assert col.decks.id_for_name(seed_deck.DEMO_DECK_NAME) == did
    assert len(col.decks.cids(did)) == card_count
