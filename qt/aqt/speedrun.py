# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Pure helpers for the Speedrun desktop reviewer hook (Phase 3b, decision D19).

The reviewer (``aqt/reviewer.py``) owns the side effects (gating Show Answer,
logging picks); this module only decides, so the logic can be unit-tested
without Qt or a collection. Everything here fails open: anything that is not an
active, *incomplete* ``SpeedrunApplication`` scaffold lets the answer through, so
a normal card or a broken template can never trap the learner.
"""

from __future__ import annotations

from dataclasses import dataclass

# Matches anki.speedrun.notetypes.APPLICATION_NOTETYPE_NAME (the Phase 1
# contract). Only this note type gates Show Answer.
APPLICATION_NOTETYPE = "SpeedrunApplication"

# pycmd signals the SpeedrunApplication template emits (Phase 1). The template
# also sets window.speedrunScaffoldComplete + .sr-app[data-scaffold-complete].
SIGNAL_PREFIX = "speedrun:"
PICK_PREFIX = "speedrun:pick:"
SCAFFOLD_COMPLETE = "speedrun:scaffold:complete"

# The one GATE_PROBE_JS result that holds Show Answer back. Every other value
# ("complete", "absent", "unscaffolded", "error", or None from a failed eval)
# lets the answer through.
GATE_INCOMPLETE = "incomplete"

# Evaluated in the card webview the moment the learner asks for the answer, to
# classify the live scaffold from the template's own soft signals. It never
# throws, so a missing or odd DOM degrades to a non-blocking result.
GATE_PROBE_JS = """
(function () {
  try {
    if (window.speedrunScaffoldComplete === true) return "complete";
    var el = document.querySelector(".sr-app");
    if (!el) return "absent";
    if (el.getAttribute("data-scaffold-complete") === "true") return "complete";
    if (el.classList.contains("sr-app--unscaffolded")) return "unscaffolded";
    return "incomplete";
  } catch (e) {
    return "error";
  }
})();
"""


def is_application_note_type(note_type_name: str | None) -> bool:
    """True only for the gated SpeedrunApplication note type."""
    return note_type_name == APPLICATION_NOTETYPE


def gate_blocks_answer(probe_result: str | None) -> bool:
    """Whether a gate-probe result should hold Show Answer back.

    Fails open: only a definite ``incomplete`` blocks. A complete, absent, or
    unscaffolded card, an error, or ``None`` (the eval failed) all allow it.
    """
    return probe_result == GATE_INCOMPLETE


@dataclass(frozen=True)
class ScaffoldPick:
    """A single level pick parsed from a ``speedrun:pick:`` signal."""

    level: str
    correct: bool
    raw: str


def parse_pick_signal(url: str) -> ScaffoldPick | None:
    """Parse ``speedrun:pick:<level>:<correct>`` into a ScaffoldPick.

    The landed template emits ``speedrun:pick:<level>:<0|1>``. Parsing is
    defensive (trailing fields are tolerated, missing ones default), and any
    non-pick or malformed signal returns None so it is ignored rather than
    fatal.
    """
    if not url.startswith(PICK_PREFIX):
        return None
    parts = url[len(PICK_PREFIX) :].split(":")
    if not parts[0]:
        return None
    return ScaffoldPick(
        level=parts[0], correct=len(parts) > 1 and parts[1] == "1", raw=url
    )
