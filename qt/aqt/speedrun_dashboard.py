# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Speedrun: the per-deck score dashboard (Memory live, Performance and
Readiness deferred).

Mirrors :mod:`aqt.deckoptions`: a webview hosting a SvelteKit page, opened for a
deck. It is read-only, it only reads the Memory score, so unlike deck options
there is no save or pending-changes close handshake.
"""

from __future__ import annotations

import aqt
import aqt.main
from anki.decks import DeckDict
from aqt.qt import *
from aqt.utils import disable_help_button, restoreGeom, saveGeom
from aqt.webview import AnkiWebView, AnkiWebViewKind


class SpeedrunDashboardDialog(QDialog):
    "The Speedrun score dashboard for a single deck."

    TITLE = "speedrunDashboard"
    silentlyClose = True

    def __init__(self, mw: aqt.main.AnkiQt, deck: DeckDict) -> None:
        QDialog.__init__(self, mw, Qt.WindowType.Window)
        self.mw = mw
        self._deck = deck
        self._setup_ui()

    def _setup_ui(self) -> None:
        self.mw.garbage_collect_on_dialog_finish(self)
        self.setMinimumWidth(400)
        self.setMinimumHeight(400)
        disable_help_button(self)
        restoreGeom(self, self.TITLE, default_size=(900, 680))

        self.web = AnkiWebView(kind=AnkiWebViewKind.SPEEDRUN_DASHBOARD)
        self.web.load_sveltekit_page(f"speedrun-dashboard/{self._deck['id']}")
        layout = QVBoxLayout()
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self.web)
        self.setLayout(layout)
        self.setWindowTitle(f"Scores: {self._deck['name']}")
        self.show()

    def reject(self) -> None:
        self.web.cleanup()
        self.web = None  # type: ignore
        saveGeom(self, self.TITLE)
        QDialog.reject(self)


def show_speedrun_dashboard(deck: DeckDict) -> None:
    SpeedrunDashboardDialog(aqt.mw, deck)


def show_speedrun_dashboard_for_current_deck() -> None:
    if not aqt.mw.col:
        return
    show_speedrun_dashboard(aqt.mw.col.decks.current())
