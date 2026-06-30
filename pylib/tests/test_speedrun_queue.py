# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""End-to-end test for the Speedrun topic-grouped ("Learn") review queue.

Unlike test_speedrun_content (pure Python), this drives the new Rust RPC through
the built backend: it tags due review cards with AAMC taxonomy ids and asserts
the queue returns topic-contiguous blocks ordered by exam weight, leaving the
default queue untouched. Needs rsbridge, so it runs under a full build.
"""

from __future__ import annotations

# Import collection first: it pulls in the full anki package and avoids the
# circular import that hits if anki.decks is the first submodule loaded.
from anki.collection import Collection
from anki.decks import DeckId
from tests.shared import getEmptyCol

# Leaf taxonomy ids; must match rslib seed_taxonomy(). Seed exam weights in
# parentheses drive the expected block order.
KINETICS = "mcat::biomolecules::enzymes::kinetics"  # 0.18
STRUCTURE = "mcat::biomolecules::amino_acids::structure"  # 0.15
PKA = "mcat::biomolecules::amino_acids::pka_titration"  # 0.12

DEFAULT_DECK = DeckId(1)


def _add_due_review_card(col: Collection, tag: str | None) -> int:
    """Add a Basic note (optionally topic-tagged) and make its card a review
    card due today, returning the card id."""
    note = col.new_note(col.models.by_name("Basic"))
    note["Front"] = "q"
    if tag is not None:
        note.tags.append(tag)
    col.add_note(note, DEFAULT_DECK)
    card_id = note.cards()[0].id
    col.sched.set_due_date([card_id], "0")
    return card_id


def test_topic_grouped_queue_blocks_and_orders():
    col = getEmptyCol()
    try:
        topic_of: dict[int, str] = {}
        # Interleave creation order across topics so grouping is non-trivial.
        for _ in range(2):
            for tag in (PKA, KINETICS, STRUCTURE):
                topic_of[_add_due_review_card(col, tag)] = tag
        # A card with no taxonomy tag is unmapped.
        topic_of[_add_due_review_card(col, None)] = ""

        queued = col.sched.get_topic_grouped_queue(deck_id=DEFAULT_DECK, fetch_limit=0)

        # Only review cards are served; the count is the full block total.
        assert len(queued.cards) == 7
        assert queued.review_count == 7
        assert queued.new_count == 0
        assert queued.learning_count == 0

        topics = [topic_of[qc.card.id] for qc in queued.cards]

        # Each topic occupies a single contiguous run (never reappears).
        runs = [t for i, t in enumerate(topics) if i == 0 or topics[i - 1] != t]
        assert len(runs) == len(set(runs)), f"blocks not contiguous: {topics}"

        # With no FSRS state or graded history the topics are equally weak, so
        # blocks fall in exam-weight order and the unmapped block trails.
        assert runs == [KINETICS, STRUCTURE, PKA, ""], topics

        # fetch_limit caps the returned cards without changing the count.
        capped = col.sched.get_topic_grouped_queue(deck_id=DEFAULT_DECK, fetch_limit=3)
        assert len(capped.cards) == 3
        assert capped.review_count == 7

        # Additive safety: the default queue still serves the same review cards.
        default = col.sched.get_queued_cards(fetch_limit=20)
        assert default.review_count == 7
    finally:
        col.close()
