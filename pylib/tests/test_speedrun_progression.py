# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""End-to-end test for the Speedrun mastery progression (decisions D30-D32).

Drives the new RPCs through the built backend: get_speedrun_card_mode,
speedrun_record_answer, get_speedrun_progress, and the state-aware
get_topic_grouped_queue. It installs the real SpeedrunConcept/SpeedrunApplication
note types, fabricates graded revlog to clear the mastery signal, and asserts the
four-state lifecycle, the state-driven card modes, application-card suppression,
and the blocked-vs-mixed queue selection. Needs rsbridge, so it runs under a full
build.
"""

from __future__ import annotations

# Import collection first: it pulls in the full anki package and avoids the
# circular import that hits if anki.decks is the first submodule loaded.
from typing import cast

from anki.collection import Collection
from anki.decks import DeckId
from anki.models import NotetypeDict
from anki.scheduler.v3 import CardAnswer
from anki.scheduler.v3 import Scheduler as V3Scheduler
from anki.speedrun.notetypes import (
    APPLICATION_NOTETYPE_NAME,
    CONCEPT_NOTETYPE_NAME,
    install_speedrun_notetypes,
)
from anki.utils import int_time
from tests.shared import getEmptyCol

# Leaf taxonomy ids; must match rslib seed_taxonomy(). Seed weights in
# parentheses drive the blocked-phase ordering.
KINETICS = "mcat::biomolecules::enzymes::kinetics"  # 0.18
STRUCTURE = "mcat::biomolecules::amino_acids::structure"  # 0.15

DEFAULT_DECK = DeckId(1)


def _notetypes(col: Collection) -> tuple[NotetypeDict, NotetypeDict]:
    ids = install_speedrun_notetypes(col)
    concept = col.models.get(ids[CONCEPT_NOTETYPE_NAME])
    application = col.models.get(ids[APPLICATION_NOTETYPE_NAME])
    assert concept is not None and application is not None
    return concept, application


def _add_card(col: Collection, notetype: NotetypeDict, tag: str) -> int:
    """Add a note of `notetype` tagged with a taxonomy leaf id; return its card
    id. The first field is set so add_note accepts the note."""
    note = col.new_note(notetype)
    note.fields[0] = f"q {tag}"
    note.tags.append(tag)
    col.add_note(note, DEFAULT_DECK)
    return note.cards()[0].id


def _log_good_reviews(col: Collection, card_id: int, count: int) -> None:
    """Fabricate `count` Good (ease 3), Review-kind revlog entries for the card,
    enough to clear the advancement signal (>= MIN_REPS at 100% >=Good)."""
    base = int_time(1000)
    for i in range(count):
        # columns: id, cid, usn, ease, ivl, lastIvl, factor, time, type
        col.db.execute(
            "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
            " values (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            base + i,
            card_id,
            -1,
            3,
            1,
            1,
            2500,
            1000,
            1,
        )


def _set_states(col: Collection, states: dict[str, str]) -> None:
    """Write the per-topic state map directly (the same config key the backend
    reads), to set up the queue's blocked-vs-mixed selection deterministically."""
    col.set_config(
        "speedrun_topic_state",
        {topic: {"state": state, "updated_at": 0} for topic, state in states.items()},
    )


def _progress(col: Collection) -> dict[str, str]:
    sched = cast(V3Scheduler, col.sched)
    topics = sched.get_speedrun_progress(deck_id=DEFAULT_DECK)
    return {p.topic_id: p.state for p in topics}


def _served_ids(col: Collection) -> set[int]:
    sched = cast(V3Scheduler, col.sched)
    queued = sched.get_topic_grouped_queue(deck_id=DEFAULT_DECK, fetch_limit=0)
    return {qc.card.id for qc in queued.cards}


def test_card_modes_and_state_transitions():
    col = getEmptyCol()
    try:
        concept_nt, app_nt = _notetypes(col)
        concept = _add_card(col, concept_nt, KINETICS)
        application = _add_card(col, app_nt, KINETICS)

        # A fresh topic is `learning`: concept learns; application is suppressed.
        assert col.sched.get_speedrun_card_mode(card_id=concept) == "concept_learn"
        assert col.sched.get_speedrun_card_mode(card_id=application) == "none"
        assert _progress(col) == {KINETICS: "learning"}

        # Clear the concept-mode signal, then advance learning -> practicing.
        _log_good_reviews(col, concept, 3)
        assert (
            col.sched.speedrun_record_answer(card_id=concept, rating=CardAnswer.GOOD)
            == "practicing"
        )
        assert col.sched.get_speedrun_card_mode(card_id=concept) == "concept_practice"
        # Application is still suppressed below hierarchy.
        assert col.sched.get_speedrun_card_mode(card_id=application) == "none"

        # Another pass advances practicing -> hierarchy; the scaffold appears.
        assert (
            col.sched.speedrun_record_answer(card_id=concept, rating=CardAnswer.GOOD)
            == "hierarchy"
        )
        assert (
            col.sched.get_speedrun_card_mode(card_id=application)
            == "application_scaffolded"
        )
        assert _progress(col) == {KINETICS: "hierarchy"}

        # A lapse (Again) demotes one state and re-suppresses the application.
        assert (
            col.sched.speedrun_record_answer(card_id=concept, rating=CardAnswer.AGAIN)
            == "practicing"
        )
        assert col.sched.get_speedrun_card_mode(card_id=application) == "none"
        assert _progress(col) == {KINETICS: "practicing"}
    finally:
        col.close()


def test_card_context_exposes_mode_and_breadcrumb_path():
    col = getEmptyCol()
    try:
        concept_nt, _app_nt = _notetypes(col)
        concept = _add_card(col, concept_nt, KINETICS)

        # The context carries the same mode as get_speedrun_card_mode plus the
        # taxonomy hierarchy labels for the reviewer breadcrumb (root -> leaf).
        context = col.sched.get_speedrun_card_context(card_id=concept)
        assert context.mode == "concept_learn"
        assert list(context.path) == ["Biomolecules", "Enzymes", "Kinetics"]

        # A Speedrun card with no taxonomy tag has a mode but no breadcrumb.
        untagged = col.new_note(concept_nt)
        untagged.fields[0] = "q"
        col.add_note(untagged, DEFAULT_DECK)
        untagged_context = col.sched.get_speedrun_card_context(
            card_id=untagged.cards()[0].id
        )
        assert untagged_context.mode == "concept_learn"
        assert list(untagged_context.path) == []

        # A non-Speedrun card is fully neutral even with a taxonomy tag.
        basic_nt = col.models.by_name("Basic")
        assert basic_nt is not None
        basic = col.new_note(basic_nt)
        basic.fields[0] = "front"
        basic.tags.append(KINETICS)
        col.add_note(basic, DEFAULT_DECK)
        basic_context = col.sched.get_speedrun_card_context(
            card_id=basic.cards()[0].id
        )
        assert basic_context.mode == "none"
        assert list(basic_context.path) == []
    finally:
        col.close()


def test_study_queue_falls_back_on_non_speedrun_deck():
    """The merged Study button always runs the topic-grouped queue; on a deck
    with no Speedrun topics it must serve the same cards as the normal queue."""
    col = getEmptyCol()
    try:
        basic_nt = col.models.by_name("Basic")
        assert basic_nt is not None
        ids = set()
        for i in range(3):
            note = col.new_note(basic_nt)
            note.fields[0] = f"q{i}"
            col.add_note(note, DEFAULT_DECK)
            ids.add(note.cards()[0].id)

        sched = cast(V3Scheduler, col.sched)
        topic_served = {
            qc.card.id
            for qc in sched.get_topic_grouped_queue(
                deck_id=DEFAULT_DECK, fetch_limit=0
            ).cards
        }
        default_served = {
            qc.card.id for qc in sched.get_queued_cards(fetch_limit=100).cards
        }

        assert topic_served == ids, "every non-Speedrun card is served"
        assert topic_served == default_served, "same set as the default queue"
    finally:
        col.close()


def test_topic_grouped_queue_is_state_aware():
    col = getEmptyCol()
    try:
        concept_nt, app_nt = _notetypes(col)
        kinetics_concept = _add_card(col, concept_nt, KINETICS)
        kinetics_app = _add_card(col, app_nt, KINETICS)
        structure_concept = _add_card(col, concept_nt, STRUCTURE)

        # All topics fresh (learning) → blocked on the heaviest (KINETICS),
        # and the application card is suppressed below hierarchy.
        served = _served_ids(col)
        assert kinetics_concept in served
        assert kinetics_app not in served, "application suppressed below hierarchy"
        assert structure_concept not in served, "blocked: other topics wait"

        # Graduate KINETICS to hierarchy and STRUCTURE to practicing → mixed:
        # every topic's cards are served, and the application card appears.
        _set_states(col, {KINETICS: "hierarchy", STRUCTURE: "practicing"})
        served = _served_ids(col)
        assert kinetics_concept in served
        assert kinetics_app in served, "application served at hierarchy"
        assert structure_concept in served, "graduated topics are mixed in"
    finally:
        col.close()
