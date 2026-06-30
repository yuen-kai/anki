# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""The principle-first scaffold's data model and its static feedback map.

Pure Python, no ``anki`` import: this is the no-AI stand-in for application
grading (spec-study-model section 5, decision D5). The same right/wrong + cue
logic runs three times here and once in the card template's JS; keeping a tested
Python copy lets later phases reuse it for desktop pick-logging without a
second, drifting implementation.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class ScaffoldStep:
    """One level of the hierarchy the learner narrows through.

    ``level`` is the taxonomy tier ("foundation" / "category" / "topic"),
    ``options`` are the node ids offered (the correct node plus its confusable
    siblings), and ``correct`` is the one that advances to the next level.
    """

    level: str
    options: list[str]
    correct: str


@dataclass(frozen=True)
class ApplicationItem:
    """An authored scaffold item: a problem, its worked solution, and the
    foundation -> category -> topic path the learner must identify first.

    ``feedback`` maps a node id to the discriminating cue shown when that node
    is picked wrongly (it names *why* it's wrong, not just that it is).
    """

    problem_id: str
    problem: str
    solution: str
    correct_path: list[str]
    steps: list[ScaffoldStep]
    feedback: dict[str, str] = field(default_factory=dict)


class ApplicationItemError(ValueError):
    """Raised when an :class:`ApplicationItem` is structurally malformed."""


# Shown when a wrong pick has no authored cue. Authored content should cover
# every distractor, so hitting this is a content gap, not an expected path.
DEFAULT_FEEDBACK = (
    "Not the distinguishing feature here. Look again at what the problem "
    "actually pins down before choosing."
)


def known_nodes(item: ApplicationItem) -> set[str]:
    """Every node id the item legitimately refers to (path + all options)."""
    nodes: set[str] = set(item.correct_path)
    for step in item.steps:
        nodes.update(step.options)
    return nodes


def validate_application_item(item: ApplicationItem) -> None:
    """Raise :class:`ApplicationItemError` if ``item`` can't drive the chooser.

    Checks the invariants the template JS relies on: a non-empty correct path
    that lines up with the steps, every step actually offering its own correct
    node, and a feedback map that only references nodes the learner can see.
    """
    if not item.correct_path:
        raise ApplicationItemError(f"{item.problem_id}: correct_path is empty")

    if len(item.steps) != len(item.correct_path):
        raise ApplicationItemError(
            f"{item.problem_id}: {len(item.steps)} steps but "
            f"{len(item.correct_path)} path entries; they must match level for level"
        )

    nodes = known_nodes(item)

    for index, step in enumerate(item.steps):
        if step.correct not in step.options:
            raise ApplicationItemError(
                f"{item.problem_id}: level {index} ({step.level}) does not offer "
                f"its own correct node {step.correct!r}"
            )
        if len(set(step.options)) != len(step.options):
            raise ApplicationItemError(
                f"{item.problem_id}: level {index} ({step.level}) has duplicate options"
            )
        if item.correct_path[index] != step.correct:
            raise ApplicationItemError(
                f"{item.problem_id}: correct_path[{index}] {item.correct_path[index]!r} "
                f"disagrees with level {index} correct node {step.correct!r}"
            )

    for node in item.feedback:
        if node not in nodes:
            raise ApplicationItemError(
                f"{item.problem_id}: feedback references unknown node {node!r}"
            )


def resolve_pick(
    item: ApplicationItem, level: int, picked_node: str
) -> tuple[bool, str]:
    """Resolve one pick at ``level`` (0-based step index).

    Returns ``(True, "")`` for the correct node, otherwise ``(False, cue)`` with
    the discriminating-cue feedback for the picked node (or a generic fallback).
    This mirrors the template JS so picks resolve identically offline.
    """
    if level < 0 or level >= len(item.steps):
        raise IndexError(
            f"{item.problem_id}: level {level} out of range (0..{len(item.steps) - 1})"
        )

    step = item.steps[level]
    if picked_node == step.correct:
        return (True, "")
    return (False, item.feedback.get(picked_node, DEFAULT_FEEDBACK))
