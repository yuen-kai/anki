# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Speedrun: a developer demo of the bespoke study screen.

A dialog-hosted webview that loads the ``speedrun-review-demo`` SvelteKit page,
which walks every study screen, state, and animation using inline mock data. It
needs no collection and issues no study RPCs, so it is safe to open anytime as a
preview tool. Mirrors :mod:`aqt.speedrun_dashboard`.
"""

from __future__ import annotations

import aqt
import aqt.main
from aqt.qt import *
from aqt.utils import disable_help_button, restoreGeom, saveGeom
from aqt.webview import AnkiWebView, AnkiWebViewKind


class SpeedrunDemoDialog(QDialog):
    "A gallery of the Speedrun study screens driven by mock data."

    TITLE = "speedrunReviewDemo"
    silentlyClose = True

    def __init__(self, mw: aqt.main.AnkiQt) -> None:
        QDialog.__init__(self, mw, Qt.WindowType.Window)
        self.mw = mw
        self._setup_ui()

    def _setup_ui(self) -> None:
        self.mw.garbage_collect_on_dialog_finish(self)
        self.setMinimumWidth(600)
        self.setMinimumHeight(480)
        disable_help_button(self)
        restoreGeom(self, self.TITLE, default_size=(1000, 720))

        # The demo issues no backend RPCs, so it reuses the dashboard webview
        # kind rather than adding one: the kind only tags the view and gates API
        # access, which the demo never needs.
        self.web = AnkiWebView(kind=AnkiWebViewKind.SPEEDRUN_DASHBOARD)
        self.web.load_sveltekit_page("speedrun-review-demo")
        layout = QVBoxLayout()
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self.web)
        self.setLayout(layout)
        self.setWindowTitle("Speedrun: demo study screens")
        self.show()

    def reject(self) -> None:
        self.web.cleanup()
        self.web = None  # type: ignore
        saveGeom(self, self.TITLE)
        QDialog.reject(self)


def show_speedrun_demo() -> None:
    SpeedrunDemoDialog(aqt.mw)


def setup_tools_menu(mw: aqt.main.AnkiQt) -> None:
    "Add the developer demo entry to the Tools menu."
    action = QAction("Speedrun: Demo study screens", mw)
    qconnect(action.triggered, show_speedrun_demo)
    mw.form.menuTools.addAction(action)
