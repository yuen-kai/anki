# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""End-to-end test for the Speedrun honest Memory score.

Drives the new GetMemoryScore Rust RPC through the built backend: it tags cards
with AAMC taxonomy ids, fabricates graded revlog entries, and asserts the
evidence envelope and the give-up rule (spec-scores §4–§6). Needs rsbridge, so
it runs under a full build.
"""

from __future__ import annotations

# Import collection first: it pulls in the full anki package and avoids the
# circular import that hits if anki.decks is the first submodule loaded.
from anki.collection import Collection
from anki.decks import DeckId
from anki.utils import int_time
from tests.shared import getEmptyCol

# Leaf taxonomy ids; must match rslib seed_taxonomy(). Eight leaves exist, so
# covering five of them is 62.5% coverage.
STRUCTURE = "mcat::biomolecules::amino_acids::structure"
PKA = "mcat::biomolecules::amino_acids::pka_titration"
METABOLISM = "mcat::biomolecules::amino_acids::metabolism"
FOLDING = "mcat::biomolecules::proteins::folding"
KINETICS = "mcat::biomolecules::enzymes::kinetics"
TOTAL_IN_SCOPE_TOPICS = 8

DEFAULT_DECK = DeckId(1)


def _add_reviewed_card(
    col: Collection, tag: str, reviews: int, next_id: list[int]
) -> int:
    """Add a topic-tagged Basic card and fabricate `reviews` scheduling-affecting
    revlog entries for it. `next_id` keeps revlog ids globally unique."""
    note = col.new_note(col.models.by_name("Basic"))
    note["Front"] = "q"
    note.tags.append(tag)
    col.add_note(note, DEFAULT_DECK)
    card_id = note.cards()[0].id
    for _ in range(reviews):
        next_id[0] += 1
        # columns: id, cid, usn, ease, ivl, lastIvl, factor, time, type
        # ease=3 (Good) and type=1 (Review) -> counts as a graded review.
        col.db.execute(
            "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
            " values (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            next_id[0],
            card_id,
            -1,
            3,
            1,
            1,
            2500,
            1000,
            1,
        )
    return card_id


def test_memory_score_eligible_envelope():
    col = getEmptyCol()
    try:
        next_id = [int_time(1000)]
        # Five of eight topics covered (62.5%), 50 graded reviews each = 250,
        # clearing both give-up thresholds (>=200 reviews, >=50% coverage).
        for tag in (STRUCTURE, PKA, METABOLISM, FOLDING, KINETICS):
            _add_reviewed_card(col, tag, 50, next_id)

        score = col.sched.get_memory_score(deck_id=DEFAULT_DECK)

        assert not score.abstained, "deck above the give-up line is eligible"
        assert score.abstain_reason == ""
        assert score.graded_reviews == 250
        assert abs(score.coverage_pct - 5 / TOTAL_IN_SCOPE_TOPICS) < 1e-4
        # The full envelope carries a number, an interval inside [0, 1], a
        # confidence band, drivers, and a timestamp.
        assert 0.0 < score.estimate <= 1.0
        assert 0.0 <= score.range_low <= score.estimate <= score.range_high <= 1.0
        assert score.confidence in ("low", "medium", "high")
        assert len(score.reasons) >= 1
        assert any(r.startswith("coverage") for r in score.reasons)
        assert score.updated_at_secs > 0
    finally:
        col.close()


def test_memory_score_abstains_when_thin():
    col = getEmptyCol()
    try:
        next_id = [int_time(1000)]
        # Only two topics (25%) and 3 reviews each = 6: both conditions fail.
        for tag in (STRUCTURE, KINETICS):
            _add_reviewed_card(col, tag, 3, next_id)

        score = col.sched.get_memory_score(deck_id=DEFAULT_DECK)

        assert score.abstained, "thin data must abstain"
        # No number is shown when abstaining.
        assert score.estimate == 0.0
        assert score.range_low == 0.0
        assert score.range_high == 0.0
        assert score.graded_reviews == 6
        # The reason names both failed conditions and what clears them.
        assert "graded reviews" in score.abstain_reason
        assert "coverage" in score.abstain_reason
    finally:
        col.close()
