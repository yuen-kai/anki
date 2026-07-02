# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from dataclasses import dataclass

from anki.collection import OpChanges
from aqt import AnkiQt
from aqt.sound import av_player


@dataclass
class DeckBrowserContent:
    """Retained only for the ``deck_browser_will_render_content`` hook's type
    signature. The deck home is now the ``speedrun-decks`` SvelteKit page, so the
    legacy HTML renderer that populated this is gone.

    Attributes:
        tree {str} -- HTML of the deck tree section
        stats {str} -- HTML of the stats section
    """

    tree: str
    stats: str


class DeckBrowser:
    """The deck home.

    Hosts the ``speedrun-decks`` SvelteKit page in the dedicated Speedrun
    webview; all deck actions (open, options, create, delete) run through that
    page's RPCs, so the old stdHtml deck tree and its link handler are gone.
    """

    def __init__(self, mw: AnkiQt) -> None:
        self.mw = mw
        self.web = mw.speedrunWeb
        self._refresh_needed = False

    def show(self) -> None:
        av_player.stop_and_clear_queue()
        self.refresh()

    def refresh(self) -> None:
        # (re)load the SvelteKit deck home so the decks list and To Do counts
        # stay current
        self.web.load_sveltekit_page("speedrun-decks")
        self._refresh_needed = False

    def refresh_if_needed(self) -> None:
        if self._refresh_needed:
            self.refresh()

    def op_executed(
        self, changes: OpChanges, handler: object | None, focused: bool
    ) -> bool:
        if changes.study_queues and handler is not self:
            self._refresh_needed = True

        if focused:
            self.refresh_if_needed()

        return self._refresh_needed
