# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
from __future__ import annotations

import html
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

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
from aqt.toolbar import BottomBar
from aqt.utils import askUserDialog, openLink, shortcut, tooltip, tr


class OverviewBottomBar:
    def __init__(self, overview: Overview) -> None:
        self.overview = overview


@dataclass
class OverviewContent:
    """Stores sections of HTML content that the overview will be
    populated with.

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
    "Deck overview."

    def __init__(self, mw: aqt.AnkiQt) -> None:
        self.mw = mw
        self.web = mw.web
        self.bottom = BottomBar(mw, mw.bottomWeb)
        self._refresh_needed = False

    def show(self) -> None:
        av_player.stop_and_clear_queue()
        # the bespoke study screen owns the whole window: hide the top/bottom
        # bars and the reviewer/overview webview, and render into the dedicated
        # Speedrun webview (mirrors DeckBrowser). Called on every entry to the
        # overview state, so it also re-hides the bars idempotently on return
        # from the reviewer.
        self.mw.toolbarWeb.hide()
        self.mw.bottomWeb.hide()
        self.mw.web.hide()
        self.mw.speedrunWeb.show()
        self.mw.setStateShortcuts(self._shortcutKeys())
        self.refresh()

    def refresh(self) -> None:
        # the study screen is a SvelteKit page hosted in the Speedrun webview;
        # (re)load it for the current deck so its counts and scores stay current
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

    # Handlers
    ############################################################

    def _linkHandler(self, url: str) -> bool:
        if url in {"study", "learn", "practice"}:
            # One Study button runs the mastery progression: the state-aware
            # topic-grouped queue, which falls back to the normal queue when the
            # deck has no Speedrun topics (decisions D26, D30-D32, supersedes the
            # Learn/Practice split in D20). "learn"/"practice" stay as aliases so
            # older callers still start a session.
            self._start_session()
        elif url == "anki":
            print("anki menu")
        elif url == "opts":
            display_options_for_deck(self.mw.col.decks.current())
        elif url == "scores":
            # Speedrun: open this deck's score dashboard from inside the deck,
            # rather than the Tools menu (decision D34). Import the submodule via
            # `from` so the local name is `speedrun_dashboard`, not `aqt` (a bare
            # `import aqt.speedrun_dashboard` rebinds `aqt` for this whole scope).
            from aqt import speedrun_dashboard

            speedrun_dashboard.show_speedrun_dashboard(self.mw.col.decks.current())
        elif url == "cram":
            aqt.dialogs.open("FilteredDeckConfigDialog", self.mw)
        elif url == "refresh":
            self.rebuild_current_filtered_deck()
        elif url == "empty":
            self.empty_current_filtered_deck()
        elif url == "decks":
            self.mw.moveToState("deckBrowser")
        elif url == "review":
            openLink(f"{aqt.appShared}info/{self.sid}?v={self.sidVer}")
        elif url in {"studymore", "customStudy"}:
            self.onStudyMore()
        elif url == "unbury":
            self.on_unbury()
        elif url == "description":
            self.edit_description()
        elif url.lower().startswith("http"):
            openLink(url)
        return False

    def _start_session(self) -> None:
        """Start study, running the Speedrun mastery progression.

        Arms the reviewer's state-aware topic-grouped queue for the current
        deck. It serves the blocked->mixed progression for Speedrun topics and
        falls back to the normal queue when the deck has none, so one button
        covers both (decisions D30-D32).
        """
        deck_id = self.mw.col.decks.get_current_id()
        self.mw.reviewer.set_speedrun_learn_deck(deck_id)
        self.mw.col.startTimebox()
        self.mw.moveToState("review")
        if self.mw.state == "overview":
            # No cards to serve: review bounced straight back to the overview.
            tooltip(tr.studying_no_cards_are_due_yet())

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

    # HTML
    ############################################################

    def _renderPage(self) -> None:
        deck = self.mw.col.decks.current()
        self.sid = deck.get("sharedFrom")
        if self.sid:
            self.sidVer = deck.get("ver", None)
            shareLink = '<a class=smallLink href="review">Reviews and Updates</a>'
        else:
            shareLink = ""
        if self.mw.col.sched._is_finished():
            self._show_finished_screen()
            return
        content = OverviewContent(
            deck=deck["name"],
            shareLink=shareLink,
            desc=self._desc(deck),
            table=self._table(),
        )
        gui_hooks.overview_will_render_content(self, content)
        content.deck = html.escape(content.deck)
        self.web.stdHtml(
            self._body % content.__dict__,
            css=["css/overview.css"],
            js=["js/vendor/jquery.min.js"],
            context=self,
        )

    def _show_finished_screen(self) -> None:
        self.web.load_sveltekit_page("congrats")

    def _desc(self, deck: dict[str, Any]) -> str:
        if deck["dyn"]:
            desc = tr.studying_this_is_a_special_deck_for()
            desc += f" {tr.studying_cards_will_be_automatically_returned_to()}"
            desc += f" {tr.studying_deleting_this_deck_from_the_deck()}"
        else:
            desc = deck.get("desc", "")
            if deck.get("md", False):
                desc = self.mw.col.render_markdown(desc)
        if not desc:
            return "<p>"
        if deck["dyn"]:
            dyn = "dyn"
        else:
            dyn = ""
        return f'<div class="descfont descmid description {dyn}">{desc}</div>'

    def _table(self) -> str:
        counts = list(self.mw.col.sched.counts())
        but = self.mw.button

        def count_cell(label: str, klass: str, count: int) -> str:
            return f"""
<div class="study-count">
    <span class="study-count__n {klass}">{count}</span>
    <span class="study-count__label">{label}</span>
</div>"""

        counts_html = (
            count_cell(tr.actions_new(), "new-count", counts[0])
            + count_cell(tr.scheduling_learning(), "learn-count", counts[1])
            + count_cell(tr.studying_to_review(), "review-count", counts[2])
        )

        # One Study button: the progression unifies blocked->mixed, so the
        # learner no longer picks a mode (decision D30, supersedes D20). Autofocus
        # keeps Enter starting the session.
        study = but("study", tr.studying_study(), id="study", extra=" autofocus")
        return f"""
<div class="study">
    <div class="study-counts">{counts_html}</div>
    {study}
</div>"""

    _body = """
<center>
<h3>%(deck)s</h3>
%(shareLink)s
%(desc)s
%(table)s
</center>
"""

    def edit_description(self) -> None:
        DeckDescriptionDialog(self.mw)

    # Bottom area
    ######################################################################

    def _renderBottom(self) -> None:
        links = [
            ["O", "opts", tr.actions_options()],
            # Speedrun: this deck's Memory / Performance / Readiness dashboard.
            ["", "scores", tr.studying_scores()],
        ]
        is_dyn = self.mw.col.decks.current()["dyn"]
        if is_dyn:
            links.append(["R", "refresh", tr.actions_rebuild()])
            links.append(["E", "empty", tr.studying_empty()])
        else:
            links.append(["C", "studymore", tr.actions_custom_study()])
            # links.append(["F", "cram", _("Filter/Cram")])
        if self.mw.col.sched.have_buried():
            links.append(["U", "unbury", tr.studying_unbury()])
        if not is_dyn:
            links.append(["", "description", tr.scheduling_description()])
        link_handler = gui_hooks.overview_will_render_bottom(
            self._linkHandler,
            links,
        )
        if not callable(link_handler):
            link_handler = self._linkHandler
        buf = ""
        for b in links:
            if b[0]:
                b[0] = tr.actions_shortcut_key(val=shortcut(b[0]))
            buf += """
<button title="%s" onclick='pycmd("%s")'>%s</button>""" % tuple(b)
        self.bottom.draw(
            buf=buf,
            link_handler=link_handler,
            web_context=OverviewBottomBar(self),
        )

    # Studying more
    ######################################################################

    def onStudyMore(self) -> None:
        import aqt.customstudy

        aqt.customstudy.CustomStudy.fetch_data_and_show(self.mw)
