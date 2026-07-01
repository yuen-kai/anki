# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""End-to-end tests for the Speedrun Performance and Readiness scores (D33).

Drives the new GetPerformanceScore / GetReadinessScore Rust RPCs through the
built backend: it installs the real SpeedrunApplication note type, tags cards
with AAMC taxonomy ids, fabricates graded revlog (ease 3 = correct, ease 1 =
wrong), and asserts the shared evidence envelope, the give-up rule, and the
ratio/points format. Needs rsbridge, so it runs under a full build.
"""

from __future__ import annotations

# Import collection first: it pulls in the full anki package and avoids the
# circular import that hits if anki.decks is the first submodule loaded.
from anki.collection import Collection
from anki.decks import DeckId
from anki.models import NotetypeDict
from anki.speedrun.notetypes import APPLICATION_NOTETYPE_NAME, install_speedrun_notetypes
from anki.utils import int_time
from tests.shared import getEmptyCol

# Leaf taxonomy ids; must match rslib seed_taxonomy(). Eight leaves exist.
STRUCTURE = "mcat::biomolecules::amino_acids::structure"
METABOLISM = "mcat::biomolecules::amino_acids::metabolism"
FOLDING = "mcat::biomolecules::proteins::folding"
KINETICS = "mcat::biomolecules::enzymes::kinetics"
FOUR_TOPICS = (STRUCTURE, METABOLISM, FOLDING, KINETICS)  # 4 of 8 = 50% coverage

DEFAULT_DECK = DeckId(1)


def _application_notetype(col: Collection) -> NotetypeDict:
    ids = install_speedrun_notetypes(col)
    nt = col.models.get(ids[APPLICATION_NOTETYPE_NAME])
    assert nt is not None
    return nt


def _add_application_card(
    col: Collection,
    notetype: NotetypeDict,
    tag: str,
    attempts: int,
    correct: int,
    next_id: list[int],
) -> int:
    """Add an application card tagged `tag` with `attempts` graded reviews, the
    first `correct` of them Good (ease 3), the rest Again (ease 1)."""
    note = col.new_note(notetype)
    note.fields[0] = f"q {tag}"
    note.tags.append(tag)
    col.add_note(note, DEFAULT_DECK)
    card_id = note.cards()[0].id
    for i in range(attempts):
        next_id[0] += 1
        ease = 3 if i < correct else 1
        # columns: id, cid, usn, ease, ivl, lastIvl, factor, time, type
        col.db.execute(
            "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
            " values (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            next_id[0],
            card_id,
            -1,
            ease,
            1,
            1,
            2500,
            1000,
            1,
        )
    return card_id


def test_performance_eligible_envelope():
    col = getEmptyCol()
    try:
        nt = _application_notetype(col)
        next_id = [int_time(1000)]
        # 4 of 8 topics (50%), 8 attempts each (32), 6 correct each -> acc 0.75.
        for tag in FOUR_TOPICS:
            _add_application_card(col, nt, tag, 8, 6, next_id)

        score = col.sched.get_performance_score(deck_id=DEFAULT_DECK)

        assert not score.abstained, "deck above the give-up line is eligible"
        assert score.abstain_reason == ""
        assert score.graded_reviews == 32
        assert score.format == "ratio"
        assert 0.0 < score.estimate <= 1.0
        assert 0.0 <= score.range_low <= score.estimate <= score.range_high <= 1.0
        assert any(r.startswith("coverage") for r in score.reasons)
    finally:
        col.close()


def test_performance_abstains_when_thin():
    col = getEmptyCol()
    try:
        nt = _application_notetype(col)
        next_id = [int_time(1000)]
        # 4 topics but only 3 attempts each = 12 < 30.
        for tag in FOUR_TOPICS:
            _add_application_card(col, nt, tag, 3, 2, next_id)

        score = col.sched.get_performance_score(deck_id=DEFAULT_DECK)

        assert score.abstained, "thin application data must abstain"
        assert score.estimate == 0.0
        assert score.graded_reviews == 12
        assert "application attempts" in score.abstain_reason
    finally:
        col.close()


def test_readiness_projects_onto_scale():
    col = getEmptyCol()
    try:
        nt = _application_notetype(col)
        next_id = [int_time(1000)]
        for tag in FOUR_TOPICS:
            _add_application_card(col, nt, tag, 8, 6, next_id)

        score = col.sched.get_readiness_score(deck_id=DEFAULT_DECK)

        assert not score.abstained
        assert score.format == "points"
        # A whole score inside a valid 472-528 band.
        assert 472.0 <= score.range_low <= score.estimate <= score.range_high <= 528.0
        assert score.estimate == round(score.estimate)
        # The readiness display leads with coverage ("Projected X, covered Y%…").
        assert score.reasons and score.reasons[0].startswith("covered")
    finally:
        col.close()


def test_readiness_abstains_when_performance_abstains():
    col = getEmptyCol()
    try:
        nt = _application_notetype(col)
        next_id = [int_time(1000)]
        # Thin: Performance abstains, so Readiness has nothing to project.
        for tag in (STRUCTURE, KINETICS):
            _add_application_card(col, nt, tag, 3, 2, next_id)

        score = col.sched.get_readiness_score(deck_id=DEFAULT_DECK)

        assert score.abstained
        assert score.estimate == 0.0
        assert score.format == "points"
        assert "performance" in score.abstain_reason.lower()
    finally:
        col.close()
