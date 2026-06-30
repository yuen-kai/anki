# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Pure-Python tests for Speedrun's no-AI content layer.

These cover the static feedback map and the authored seed only. They import
``anki.speedrun.feedback`` and ``anki.speedrun.seed_content`` as a namespace
package and never ``import anki``, so they run with no rsbridge / Rust build.

``anki`` has no ``__init__.py`` in the source tree (it is generated at build
time), so it imports as a PEP 420 namespace package: pulling in
``anki.speedrun.feedback`` runs no backend code. We only need ``pylib`` on the
path, which we add below so the test works however it is launched.

Run it without a build via either::

    python3 pylib/tests/test_speedrun_content.py
    cd pylib/tests && python3 -m unittest test_speedrun_content

(The repo's ``pylib/tests/__init__.py`` imports the Rust backend, so a plain
``pytest pylib/tests/...`` needs a full build to collect; see the chat report.)
"""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from anki.speedrun import seed_content
from anki.speedrun.feedback import (
    DEFAULT_FEEDBACK,
    ApplicationItem,
    ApplicationItemError,
    ScaffoldStep,
    known_nodes,
    resolve_pick,
    validate_application_item,
)


def _valid_item() -> ApplicationItem:
    """A minimal well-formed item, used as the baseline to mutate from."""
    return ApplicationItem(
        problem_id="probe",
        problem="problem text",
        solution="solution text",
        correct_path=["F", "C"],
        steps=[
            ScaffoldStep("foundation", ["F", "F2"], "F"),
            ScaffoldStep("category", ["C", "C2"], "C"),
        ],
        feedback={"F2": "wrong foundation cue", "C2": "wrong category cue"},
    )


class ValidateTest(unittest.TestCase):
    def test_valid_item_passes(self):
        # Should not raise.
        validate_application_item(_valid_item())

    def test_empty_correct_path_rejected(self):
        item = _valid_item()
        bad = ApplicationItem(
            item.problem_id,
            item.problem,
            item.solution,
            [],
            item.steps,
            item.feedback,
        )
        with self.assertRaises(ApplicationItemError):
            validate_application_item(bad)

    def test_step_missing_its_correct_node_rejected(self):
        bad = _valid_item()
        bad.steps[1] = ScaffoldStep("category", ["C2", "C3"], "C")  # "C" not offered
        with self.assertRaises(ApplicationItemError):
            validate_application_item(bad)

    def test_path_step_length_mismatch_rejected(self):
        bad = _valid_item()
        bad.correct_path.append("extra")
        with self.assertRaises(ApplicationItemError):
            validate_application_item(bad)

    def test_path_disagrees_with_step_correct_rejected(self):
        bad = _valid_item()
        bad.correct_path[0] = "F2"  # offered, but not the level's correct node
        with self.assertRaises(ApplicationItemError):
            validate_application_item(bad)

    def test_feedback_for_unknown_node_rejected(self):
        bad = _valid_item()
        bad.feedback["ghost"] = "references a node nobody can pick"
        with self.assertRaises(ApplicationItemError):
            validate_application_item(bad)

    def test_duplicate_options_rejected(self):
        bad = _valid_item()
        bad.steps[0] = ScaffoldStep("foundation", ["F", "F", "F2"], "F")
        with self.assertRaises(ApplicationItemError):
            validate_application_item(bad)

    def test_error_is_a_value_error(self):
        self.assertTrue(issubclass(ApplicationItemError, ValueError))


class ResolvePickTest(unittest.TestCase):
    def test_correct_pick_advances_with_no_feedback(self):
        item = _valid_item()
        ok, msg = resolve_pick(item, 0, "F")
        self.assertTrue(ok)
        self.assertEqual(msg, "")

    def test_wrong_pick_returns_its_authored_cue(self):
        item = _valid_item()
        ok, msg = resolve_pick(item, 0, "F2")
        self.assertFalse(ok)
        self.assertEqual(msg, "wrong foundation cue")

    def test_wrong_pick_per_level(self):
        item = _valid_item()
        ok, msg = resolve_pick(item, 1, "C2")
        self.assertFalse(ok)
        self.assertEqual(msg, "wrong category cue")

    def test_wrong_pick_without_authored_cue_uses_default(self):
        item = _valid_item()
        ok, msg = resolve_pick(item, 0, "F-unlabelled")
        self.assertFalse(ok)
        self.assertEqual(msg, DEFAULT_FEEDBACK)

    def test_out_of_range_level_raises(self):
        item = _valid_item()
        with self.assertRaises(IndexError):
            resolve_pick(item, 5, "F")


class SeedContentTest(unittest.TestCase):
    def test_has_minimum_seed_volume(self):
        self.assertGreaterEqual(len(seed_content.CONTRASTING_CASES), 2)
        self.assertGreaterEqual(len(seed_content.APPLICATION_ITEMS), 2)

    def test_every_application_item_validates(self):
        for item in seed_content.APPLICATION_ITEMS:
            with self.subTest(item=item.problem_id):
                validate_application_item(item)

    def test_application_items_are_three_level_scaffolds(self):
        for item in seed_content.APPLICATION_ITEMS:
            with self.subTest(item=item.problem_id):
                self.assertEqual(
                    [s.level for s in item.steps], ["foundation", "category", "topic"]
                )

    def test_every_distractor_has_an_authored_cue(self):
        # The quality bar: wrong picks must name the discriminating cue, so the
        # generic fallback should never fire on seeded content.
        for item in seed_content.APPLICATION_ITEMS:
            for index, step in enumerate(item.steps):
                for node in step.options:
                    if node == step.correct:
                        continue
                    _, msg = resolve_pick(item, index, node)
                    with self.subTest(item=item.problem_id, node=node):
                        self.assertIn(node, item.feedback)
                        self.assertNotEqual(msg, DEFAULT_FEEDBACK)
                        self.assertTrue(msg.strip())

    def test_problem_ids_unique(self):
        ids = [i.problem_id for i in seed_content.APPLICATION_ITEMS]
        self.assertEqual(len(ids), len(set(ids)))

    def test_contrasting_cases_are_populated(self):
        for case in seed_content.CONTRASTING_CASES:
            with self.subTest(topic=case.topic_id):
                for text in (
                    case.case_a,
                    case.case_b,
                    case.similarity_prompt,
                    case.concept,
                ):
                    self.assertTrue(text.strip())
                # Two genuinely different surfaces, not a near-duplicate pair.
                self.assertNotEqual(case.case_a, case.case_b)

    def test_every_node_in_play_has_a_label(self):
        for item in seed_content.APPLICATION_ITEMS:
            for node in known_nodes(item):
                with self.subTest(node=node):
                    self.assertTrue(seed_content.node_label(node).strip())

    def test_node_label_fallback(self):
        self.assertEqual(
            seed_content.node_label("mcat::biomolecules::enzymes::regulation"),
            "Regulation",
        )
        self.assertEqual(
            seed_content.node_label("totally::unknown_node"), "Unknown node"
        )

    def test_seed_uses_the_contract_topic_ids(self):
        # Guard the exact taxonomy ids later phases depend on.
        self.assertEqual(
            seed_content.TOPIC_ENZYME_INHIBITION,
            "mcat::biomolecules::enzymes::inhibition",
        )
        self.assertEqual(
            seed_content.TOPIC_AA_PKA_TITRATION,
            "mcat::biomolecules::amino_acids::pka_titration",
        )
        self.assertEqual(seed_content.FOUNDATION_BIOMOLECULES, "mcat::biomolecules")


if __name__ == "__main__":
    unittest.main(verbosity=2)
