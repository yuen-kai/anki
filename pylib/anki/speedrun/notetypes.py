# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""The two Speedrun note types and an idempotent installer (decision D19).

Authoring lives in notes, not in a bespoke reviewer, so the contrasting-cases
and scaffold interactions sync and render unchanged on desktop Qt and AnkiDroid.
This module is the only one in the package that touches a collection; it is not
unit-tested this phase (it needs the Rust backend). The card behaviour itself is
plain HTML/CSS/JS read from ``templates/`` and makes no network or model calls.
"""

from __future__ import annotations

import json
from pathlib import Path

import anki.collection
from anki.models import NotetypeDict, NotetypeId
from anki.notes import NoteId
from anki.speedrun import seed_content
from anki.speedrun.feedback import ApplicationItem, validate_application_item

CONCEPT_NOTETYPE_NAME = "SpeedrunConcept"
APPLICATION_NOTETYPE_NAME = "SpeedrunApplication"

# Field order is part of the contract; later phases address fields by name.
CONCEPT_FIELDS = ["CaseA", "CaseB", "SimilarityPrompt", "Concept"]
APPLICATION_FIELDS = ["Problem", "Solution", "CorrectPath", "Steps", "Feedback"]

SEED_DECK_NAME = "Speedrun"
SEED_TAG = "speedrun::seed"

_TEMPLATES_DIR = Path(__file__).parent / "templates"


def _asset(name: str) -> str:
    return (_TEMPLATES_DIR / name).read_text(encoding="utf-8")


def _inline_script(html: str, js: str) -> str:
    return f"{html.rstrip()}\n\n<script>\n{js.strip()}\n</script>\n"


def _build(
    col: anki.collection.Collection,
    name: str,
    fields: list[str],
    card_name: str,
    qfmt: str,
    afmt: str,
    css: str,
) -> NotetypeDict:
    notetype = col.models.new(name)
    for field_name in fields:
        col.models.add_field(notetype, col.models.new_field(field_name))
    template = col.models.new_template(card_name)
    template["qfmt"] = qfmt
    template["afmt"] = afmt
    col.models.add_template(notetype, template)
    notetype["css"] = css
    return notetype


def concept_notetype(col: anki.collection.Collection) -> NotetypeDict:
    """Build (without saving) the SpeedrunConcept note type."""
    js = _asset("concept.js")
    # The back includes {{FrontSide}}, so the front's <script> carries over.
    return _build(
        col,
        CONCEPT_NOTETYPE_NAME,
        CONCEPT_FIELDS,
        "Concept",
        qfmt=_inline_script(_asset("concept_front.html"), js),
        afmt=_asset("concept_back.html"),
        css=_asset("concept.css"),
    )


def application_notetype(col: anki.collection.Collection) -> NotetypeDict:
    """Build (without saving) the SpeedrunApplication note type."""
    js = _asset("application.js")
    # The back is standalone (no {{FrontSide}}), so it needs its own <script>.
    return _build(
        col,
        APPLICATION_NOTETYPE_NAME,
        APPLICATION_FIELDS,
        "Scaffold",
        qfmt=_inline_script(_asset("application_front.html"), js),
        afmt=_inline_script(_asset("application_back.html"), js),
        css=_asset("application.css"),
    )


def install_speedrun_notetypes(
    col: anki.collection.Collection,
) -> dict[str, NotetypeId]:
    """Create the two note types if absent; return ``name -> id`` for both.

    Idempotent: an existing note type with the same name is left untouched and
    its id returned, so re-running on a synced collection is safe.
    """
    builders = {
        CONCEPT_NOTETYPE_NAME: concept_notetype,
        APPLICATION_NOTETYPE_NAME: application_notetype,
    }
    result: dict[str, NotetypeId] = {}
    for name, builder in builders.items():
        existing = col.models.id_for_name(name)
        if existing is not None:
            result[name] = existing
            continue
        col.models.add_dict(builder(col))
        new_id = col.models.id_for_name(name)
        assert new_id is not None
        result[name] = new_id
    return result


# --- Seed content -> note fields -------------------------------------------


def concept_fields(case: seed_content.ContrastingCase) -> dict[str, str]:
    """Map a ContrastingCase onto SpeedrunConcept fields."""
    return {
        "CaseA": case.case_a,
        "CaseB": case.case_b,
        "SimilarityPrompt": case.similarity_prompt,
        "Concept": case.concept,
    }


def _embed_json(value: object) -> str:
    """Serialize to JSON safe to embed in an HTML ``<script>`` block.

    ``json.dumps`` escapes the charset but not a literal ``</script>``. Since
    ``<`` only ever appears inside JSON string values, replacing it with the
    ``\\u003c`` escape neutralizes any breakout and ``JSON.parse`` restores it.
    """
    return json.dumps(value).replace("<", "\\u003c")


def application_fields(item: ApplicationItem) -> dict[str, str]:
    """Map an ApplicationItem onto SpeedrunApplication fields.

    CorrectPath/Steps/Feedback are JSON strings; Steps carries each option's
    display label so the template never has to know the taxonomy. ``<`` is
    escaped so the JSON embeds safely in the template's ``<script>`` blocks even
    if a label or cue ever contains ``</script>``.
    """
    validate_application_item(item)
    steps = [
        {
            "level": step.level,
            "options": [
                {"id": node, "label": seed_content.node_label(node)}
                for node in step.options
            ],
            "correct": step.correct,
        }
        for step in item.steps
    ]
    return {
        "Problem": item.problem,
        "Solution": item.solution,
        "CorrectPath": _embed_json(item.correct_path),
        "Steps": _embed_json(steps),
        "Feedback": _embed_json(item.feedback),
    }


def add_seed_notes(col: anki.collection.Collection) -> list[NoteId]:
    """Install the note types and load the hand-authored seed (idempotent).

    Seed notes are tagged with ``speedrun::seed`` and their taxonomy node id (a
    valid hierarchical Anki tag), so they map onto the topic tree and can be
    re-run without creating duplicates. Returns the ids of notes created on this
    call (empty if everything was already present).
    """
    ids = install_speedrun_notetypes(col)
    deck_id = col.decks.id(SEED_DECK_NAME)
    created: list[NoteId] = []

    concept_nt = col.models.get(ids[CONCEPT_NOTETYPE_NAME])
    assert concept_nt is not None
    for case in seed_content.CONTRASTING_CASES:
        marker = f"{SEED_TAG}::{case.topic_id}"
        if col.find_notes(f'note:{CONCEPT_NOTETYPE_NAME} "tag:{marker}"'):
            continue
        note = col.new_note(concept_nt)
        for name, value in concept_fields(case).items():
            note[name] = value
        note.add_tag(SEED_TAG)
        note.add_tag(marker)
        note.add_tag(case.topic_id)
        col.add_note(note, deck_id)
        created.append(note.id)

    application_nt = col.models.get(ids[APPLICATION_NOTETYPE_NAME])
    assert application_nt is not None
    for item in seed_content.APPLICATION_ITEMS:
        marker = f"{SEED_TAG}::{item.problem_id}"
        if col.find_notes(f'"tag:{marker}"'):
            continue
        note = col.new_note(application_nt)
        for name, value in application_fields(item).items():
            note[name] = value
        topic_id = item.correct_path[-1]
        note.add_tag(SEED_TAG)
        note.add_tag(marker)
        note.add_tag(topic_id)
        col.add_note(note, deck_id)
        created.append(note.id)

    return created
