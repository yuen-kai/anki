# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Unit tests for the Speedrun reviewer hook's pure decision logic (Phase 3b).

The gate and pick parser are deliberately Qt-free so they can be tested without
a running app. The load-bearing property is that the gate *fails open*: only a
live, incomplete SpeedrunApplication scaffold ever holds Show Answer back; a
normal card or a broken template never does.
"""

from __future__ import annotations

import pytest

from aqt.reviewer import Reviewer
from aqt.speedrun import (
    APPLICATION_NOTETYPE,
    GATE_INCOMPLETE,
    SCAFFOLD_COMPLETE,
    ScaffoldPick,
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
