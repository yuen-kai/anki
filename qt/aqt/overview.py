# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

import aqt
import aqt.operations
from anki.collection import OpChanges
from anki.scheduler import UnburyDeck
from aqt import gui_hooks
from aqt.deckdescription import DeckDescriptionDialog
from aqt.deckoptions import display_options_for_deck
from aqt.operations.scheduling import (
    empty_filtered_deck,
    rebuild_filtered_deck,
    unbury_deck,
)
from aqt.sound import av_player
from aqt.utils import askUserDialog, tr


@dataclass
class OverviewContent:
    """Retained only for the ``overview_will_render_content`` hook's type
    signature. The overview is now the ``speedrun-study`` SvelteKit page, so the
    legacy HTML renderer that populated this is gone.

    Attributes:
        deck {str} -- Plain text deck name
        shareLink {str} -- HTML of the share link section
        desc {str} -- HTML of the deck description section
        table {str} -- HTML of the deck stats table section
    """

    deck: str
    shareLink: str
    desc: str
    table: str


class Overview:
    """Deck overview.

    The bespoke Speedrun study screen (the ``speedrun-study`` SvelteKit page)
    owns the whole window; this host just loads it and forwards the secondary
    deck actions its overflow menu triggers via ``speedrun_overview_action``.
    """

    def __init__(self, mw: aqt.AnkiQt) -> None:
        self.mw = mw
        self.web = mw.web
        self._refresh_needed = False

    def show(self) -> None:
        av_player.stop_and_clear_queue()
        # The study screen owns the whole window: hide the top/bottom bars and
        # the reviewer/overview webview, and render into the dedicated Speedrun
        # webview (mirrors DeckBrowser). Called on every entry to the overview
        # state, so it also re-hides the bars idempotently on return.
        self.mw.toolbarWeb.hide()
        self.mw.bottomWeb.hide()
        self.mw.web.hide()
        self.mw.speedrunWeb.show()
        self.mw.setStateShortcuts(self._shortcutKeys())
        self.refresh()

    def refresh(self) -> None:
        # The study screen is a SvelteKit page hosted in the Speedrun webview;
        # (re)load it for the current deck so its counts and scores stay current.
        self._refresh_needed = False
        did = self.mw.col.decks.get_current_id()
        self.mw.speedrunWeb.load_sveltekit_page(f"speedrun-study/{did}")
        gui_hooks.overview_did_refresh(self)

    def refresh_if_needed(self) -> None:
        if self._refresh_needed:
            self.refresh()

    def op_executed(
        self, changes: OpChanges, handler: object | None, focused: bool
    ) -> bool:
        if changes.study_queues:
            self._refresh_needed = True

        if focused:
            self.refresh_if_needed()

        return self._refresh_needed

    # Secondary actions (driven from the study screen's overflow menu)
    ############################################################

    def _shortcutKeys(self) -> list[tuple[str, Callable]]:
        return [
            ("o", lambda: display_options_for_deck(self.mw.col.decks.current())),
            ("r", self.rebuild_current_filtered_deck),
            ("e", self.empty_current_filtered_deck),
            ("c", self.onCustomStudyKey),
            ("u", self.on_unbury),
        ]

    def _current_deck_is_filtered(self) -> int:
        return self.mw.col.decks.current()["dyn"]

    def rebuild_current_filtered_deck(self) -> None:
        rebuild_filtered_deck(
            parent=self.mw, deck_id=self.mw.col.decks.selected()
        ).run_in_background()

    def empty_current_filtered_deck(self) -> None:
        empty_filtered_deck(
            parent=self.mw, deck_id=self.mw.col.decks.selected()
        ).run_in_background()

    def onCustomStudyKey(self) -> None:
        if not self._current_deck_is_filtered():
            self.onStudyMore()

    def on_unbury(self) -> None:
        mode = UnburyDeck.Mode.ALL
        info = self.mw.col.sched.congratulations_info()
        if info.have_sched_buried and info.have_user_buried:
            opts = [
                tr.studying_manually_buried_cards(),
                tr.studying_buried_siblings(),
                tr.studying_all_buried_cards(),
                tr.actions_cancel(),
            ]

            diag = askUserDialog(tr.studying_what_would_you_like_to_unbury(), opts)
            diag.setDefault(0)
            ret = diag.run()
            if ret == opts[0]:
                mode = UnburyDeck.Mode.USER_ONLY
            elif ret == opts[1]:
                mode = UnburyDeck.Mode.SCHED_ONLY
            elif ret == opts[3]:
                return

        unbury_deck(
            parent=self.mw, deck_id=self.mw.col.decks.get_current_id(), mode=mode
        ).run_in_background()

    onUnbury = on_unbury

    def edit_description(self) -> None:
        DeckDescriptionDialog(self.mw)

    def onStudyMore(self) -> None:
        import aqt.customstudy

        aqt.customstudy.CustomStudy.fetch_data_and_show(self.mw)
