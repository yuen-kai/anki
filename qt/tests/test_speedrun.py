# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Unit tests for the Speedrun reviewer hook's pure decision logic.

The gate, pick parser, card-mode inject builder, and answer→rating mapping are
deliberately backend-free so they can be tested without a collection. Two
load-bearing properties: the gate *fails open* (only a live, incomplete
SpeedrunApplication scaffold ever holds Show Answer back) and the card-mode
injection is *additive* (only a known Speedrun mode injects anything, so a
normal card and normal review are never touched).
"""

from __future__ import annotations

import pytest

# CardAnswer is re-exported by aqt.reviewer; importing it from there (rather than
# anki.scheduler.v3 directly) keeps aqt first in the import order, dodging a
# pre-existing anki.scheduler circular import when this module is run alone.
from aqt.reviewer import CardAnswer, Reviewer, V3CardInfo
from aqt.speedrun import (
    APPLICATION_NOTETYPE,
    CARD_MODE_NONE,
    CARD_MODES,
    GATE_INCOMPLETE,
    SCAFFOLD_COMPLETE,
    ScaffoldPick,
    card_mode_inject_script,
    gate_blocks_answer,
    is_application_note_type,
    parse_pick_signal,
)


class TestGateAppliesOnlyToApplicationCards:
    def test_application_note_type_is_gated(self) -> None:
        assert is_application_note_type(APPLICATION_NOTETYPE) is True

    @pytest.mark.parametrize(
        "name", ["SpeedrunConcept", "Basic", "Cloze", "", "speedrunapplication", None]
    )
    def test_other_note_types_are_not_gated(self, name: str | None) -> None:
        assert is_application_note_type(name) is False


class TestGateFailsOpen:
    def test_incomplete_scaffold_blocks(self) -> None:
        assert gate_blocks_answer(GATE_INCOMPLETE) is True

    @pytest.mark.parametrize(
        "result",
        ["complete", "absent", "unscaffolded", "error", "", "INCOMPLETE", None],
    )
    def test_everything_else_lets_the_answer_through(self, result: str | None) -> None:
        # complete/absent/unscaffolded/error and a failed eval (None) must all
        # open the gate, so a bug can never trap the learner.
        assert gate_blocks_answer(result) is False


class TestParsePickSignal:
    def test_parses_a_correct_pick(self) -> None:
        assert parse_pick_signal("speedrun:pick:foundation:1") == ScaffoldPick(
            level="foundation", correct=True, raw="speedrun:pick:foundation:1"
        )

    def test_parses_a_wrong_pick(self) -> None:
        pick = parse_pick_signal("speedrun:pick:category:0")
        assert pick is not None
        assert pick.level == "category"
        assert pick.correct is False

    def test_tolerates_extra_trailing_fields(self) -> None:
        # Defensive against a future speedrun:pick:<level>:<correct>:<ms> shape.
        pick = parse_pick_signal("speedrun:pick:topic:1:980")
        assert pick is not None
        assert pick.level == "topic"
        assert pick.correct is True

    @pytest.mark.parametrize(
        "url",
        [SCAFFOLD_COMPLETE, "ans", "ease3", "speedrun:pick:", "speedrun:pick::1", ""],
    )
    def test_ignores_non_picks(self, url: str) -> None:
        assert parse_pick_signal(url) is None


class TestReviewerGateWiring:
    """The reviewer-level predicate, exercised without constructing the app."""

    def _reviewer_with_card(self, note_type_name: str | None) -> Reviewer:
        reviewer = Reviewer.__new__(Reviewer)
        reviewer._speedrun_gate_cleared = False
        if note_type_name is None:
            reviewer.card = None
        else:

            class _Card:
                def note_type(self) -> dict[str, str]:
                    return {"name": note_type_name}

            reviewer.card = _Card()  # type: ignore[assignment]
        return reviewer

    def test_normal_card_is_never_gated(self) -> None:
        assert self._reviewer_with_card("Basic")._speedrun_answer_gate_active() is False

    def test_application_card_is_gated(self) -> None:
        reviewer = self._reviewer_with_card(APPLICATION_NOTETYPE)
        assert reviewer._speedrun_answer_gate_active() is True

    def test_missing_card_is_never_gated(self) -> None:
        assert self._reviewer_with_card(None)._speedrun_answer_gate_active() is False

    def test_a_cleared_gate_does_not_reblock(self) -> None:
        reviewer = self._reviewer_with_card(APPLICATION_NOTETYPE)
        reviewer._speedrun_gate_cleared = True
        assert reviewer._speedrun_answer_gate_active() is False


class TestCardModeInjectScript:
    """The mode→inject string builder the reviewer prepends before render."""

    @pytest.mark.parametrize("mode", sorted(CARD_MODES))
    def test_known_mode_sets_the_window_variable(self, mode: str) -> None:
        # The template reads window.speedrunCardMode, so the value injected must
        # match get_speedrun_card_mode exactly (json-quoted so it can't escape
        # the tag).
        assert (
            card_mode_inject_script(mode)
            == f'<script>window.speedrunCardMode = "{mode}";</script>'
        )

    @pytest.mark.parametrize(
        "mode",
        [CARD_MODE_NONE, "", None, "garbage", "CONCEPT_LEARN", "concept", "scaffolded"],
    )
    def test_non_modes_inject_nothing(self, mode: str | None) -> None:
        # "none"/empty/None and any unknown string are a no-op, so a normal card
        # and normal review are never touched.
        assert card_mode_inject_script(mode) == ""

    def test_none_constant_is_not_a_real_mode(self) -> None:
        assert CARD_MODE_NONE not in CARD_MODES


class TestRatingFromEase:
    """The answer→record mapping fed to speedrun_record_answer (decision D32)."""

    def test_again_demotes(self) -> None:
        assert V3CardInfo.rating_from_ease(1) == CardAnswer.AGAIN

    @pytest.mark.parametrize(
        "ease,rating",
        [
            (1, CardAnswer.AGAIN),
            (2, CardAnswer.HARD),
            (3, CardAnswer.GOOD),
            (4, CardAnswer.EASY),
        ],
    )
    def test_each_ease_maps_to_its_rating(
        self, ease: int, rating: CardAnswer.Rating.V
    ) -> None:
        assert V3CardInfo.rating_from_ease(ease) == rating
