# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
from __future__ import annotations

import enum
import re
from collections.abc import Callable
from typing import Any, cast

import aqt
from anki.sync import SyncStatus
from aqt import gui_hooks, props
from aqt.qt import *
from aqt.sync import get_sync_status
from aqt.theme import theme_manager
from aqt.utils import openLink, tr
from aqt.webview import AnkiWebView, AnkiWebViewKind

# Inline icons for the app bar. currentColor so they inherit theme + accent.
_GAUGE_SVG = (
    '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" aria-hidden="true">'
    '<path d="M3.6 16.4a8.4 8.4 0 0 1 16.8 0" stroke="currentColor" stroke-width="2.3" '
    'stroke-linecap="round"/><path d="M12 16.4l5-4.1" stroke="currentColor" '
    'stroke-width="2.3" stroke-linecap="round"/><circle cx="12" cy="16.4" r="1.7" '
    'fill="currentColor"/></svg>'
)
_PLUS_SVG = (
    '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">'
    '<path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2.8" '
    'stroke-linecap="round"/></svg>'
)
_SYNC_SVG = (
    '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" aria-hidden="true">'
    '<path d="M20 11a8 8 0 0 0-14.4-4.4M4 4.6V9h4.4" stroke="currentColor" '
    'stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>'
    '<path d="M4 13a8 8 0 0 0 14.4 4.4M20 19.4V15h-4.4" stroke="currentColor" '
    'stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>'
)
_MENU_SVG = (
    '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">'
    '<path d="M4 7h16M4 12h16M4 17h16" stroke="currentColor" stroke-width="2" '
    'stroke-linecap="round"/></svg>'
)


class HideMode(enum.IntEnum):
    FULLSCREEN = 0
    ALWAYS = 1


# wrapper class for set_bridge_command()
class TopToolbar:
    def __init__(self, toolbar: Toolbar) -> None:
        self.toolbar = toolbar


# wrapper class for set_bridge_command()
class BottomToolbar:
    def __init__(self, toolbar: Toolbar) -> None:
        self.toolbar = toolbar


class ToolbarWebView(AnkiWebView):
    hide_condition: Callable[..., bool]

    def __init__(
        self, mw: aqt.AnkiQt, kind: AnkiWebViewKind = AnkiWebViewKind.DEFAULT
    ) -> None:
        AnkiWebView.__init__(self, mw, kind=kind)
        self.mw = mw
        self.setFocusPolicy(Qt.FocusPolicy.WheelFocus)
        self.disable_zoom()
        self.hidden = False
        self.hide_timer = QTimer()
        self.hide_timer.setSingleShot(True)
        self.reset_timer()

    def reset_timer(self) -> None:
        self.hide_timer.stop()
        self.hide_timer.setInterval(2000)

    def hide(self) -> None:
        self.hidden = True

    def show(self) -> None:
        self.hidden = False


class TopWebView(ToolbarWebView):
    def __init__(self, mw: aqt.AnkiQt) -> None:
        super().__init__(mw, kind=AnkiWebViewKind.TOP_TOOLBAR)
        self.web_height = 0
        qconnect(self.hide_timer.timeout, self.hide_if_allowed)

    def eventFilter(self, obj, evt):
        if handled := super().eventFilter(obj, evt):
            return handled

        # prevent collapse of both toolbars if pointer is inside one of them
        if evt.type() == QEvent.Type.Enter:
            self.reset_timer()
            self.mw.bottomWeb.reset_timer()
            return True

        return False

    def on_body_classes_need_update(self) -> None:
        super().on_body_classes_need_update()

        if self.mw.state == "review":
            if self.mw.pm.hide_top_bar():
                self.eval("""document.body.classList.remove("flat"); """)
            else:
                self.flatten()

        self.adjustHeightToFit()
        self.show()

    def _onHeight(self, qvar: int | None) -> None:
        super()._onHeight(qvar)
        if qvar:
            self.web_height = int(qvar)

    def hide_if_allowed(self) -> None:
        if self.mw.state != "review":
            return

        # Invariant: The `hide_if_allowed` method ensures that the fullscreen state is checked
        # and the menubar will be hidden if necessary
        # Note: The `eventFilter` and `_reviewState` methods in `qt/aqt/main.py` rely on this invariant
        if self.mw.fullscreen:
            self.mw.hide_menubar()

        if self.mw.pm.hide_top_bar():
            if (
                self.mw.pm.top_bar_hide_mode() == HideMode.FULLSCREEN
                and not self.mw.windowState() & Qt.WindowState.WindowFullScreen
            ):
                self.show()
                return

            self.hide()

    def hide(self) -> None:
        super().hide()

        self.hidden = True
        self.eval(
            """document.body.classList.add("hidden"); """,
        )

    def show(self) -> None:
        super().show()

        self.eval("""document.body.classList.remove("hidden"); """)

    def flatten(self) -> None:
        self.eval("""document.body.classList.add("flat"); """)

    def elevate(self) -> None:
        self.eval(
            """
            document.body.classList.remove("flat");
            document.body.style.removeProperty("background");
            """
        )

    def update_background_image(self) -> None:
        if self.mw.pm.minimalist_mode():
            return

        def set_background(computed: str) -> None:
            # remove offset from copy
            background = re.sub(r"-\d+px ", "0%", computed)
            # ensure alignment with main webview
            background = re.sub(r"\sfixed", "", background)
            # change computedStyle px value back to 100vw
            background = re.sub(r"\d+px", "100vw", background)

            self.eval(
                f"""
                    document.body.style.setProperty("background", '{background}');
                """
            )
            self.set_body_height(self.mw.web.height())

            # offset reviewer background by toolbar height
            if self.web_height:
                self.mw.web.eval(
                    f"""document.body.style.setProperty("background-position-y", "-{self.web_height}px"); """
                )

        self.mw.web.evalWithCallback(
            """window.getComputedStyle(document.body).background; """,
            set_background,
        )

    def set_body_height(self, height: int) -> None:
        self.eval(
            f"""document.body.style.setProperty("min-height", "{self.mw.web.height()}px"); """
        )

    def adjustHeightToFit(self) -> None:
        self.eval("""document.body.style.setProperty("min-height", "0px"); """)
        self.evalWithCallback("document.documentElement.offsetHeight", self._onHeight)

    def resizeEvent(self, event: QResizeEvent | None) -> None:
        super().resizeEvent(event)

        self.mw.web.evalWithCallback(
            """window.innerHeight; """,
            self.set_body_height,
        )


class BottomWebView(ToolbarWebView):
    def __init__(self, mw: aqt.AnkiQt) -> None:
        super().__init__(mw, kind=AnkiWebViewKind.BOTTOM_TOOLBAR)
        qconnect(self.hide_timer.timeout, self.hide_if_allowed)

    def eventFilter(self, obj, evt):
        if handled := super().eventFilter(obj, evt):
            return handled

        if evt.type() == QEvent.Type.Enter:
            self.reset_timer()
            self.mw.toolbarWeb.reset_timer()
            return True

        return False

    def on_body_classes_need_update(self) -> None:
        super().on_body_classes_need_update()
        if self.mw.state == "review":
            self.show()

    def animate_height(self, height: int) -> None:
        self.web_height = height

        if self.mw.pm.reduce_motion() or height == self.height():
            self.setFixedHeight(height)
        else:
            # Collapse/Expand animation
            self.setMinimumHeight(0)
            self.animation = QPropertyAnimation(
                self, cast(QByteArray, b"maximumHeight")
            )
            self.animation.setDuration(int(theme_manager.var(props.TRANSITION)))
            self.animation.setStartValue(self.height())
            self.animation.setEndValue(height)
            qconnect(self.animation.finished, lambda: self.setFixedHeight(height))
            self.animation.start()

    def hide_if_allowed(self) -> None:
        if self.mw.state != "review":
            return

        if self.mw.pm.hide_bottom_bar():
            if (
                self.mw.pm.bottom_bar_hide_mode() == HideMode.FULLSCREEN
                and not self.mw.windowState() & Qt.WindowState.WindowFullScreen
            ):
                self.show()
                return

            self.hide()

    def hide(self) -> None:
        super().hide()

        self.hidden = True
        self.animate_height(1)

    def show(self) -> None:
        super().show()

        self.hidden = False
        if self.mw.state == "review":
            # delay to account for reflow
            def cb(height: int | None):
                # "When QWebEnginePage is deleted, the callback is triggered with an invalid value"
                if height is not None:
                    self.animate_height(height)

            self.mw.progress.single_shot(
                50,
                lambda: self.evalWithCallback(
                    "document.documentElement.offsetHeight", cb
                ),
                False,
            )
        else:
            self.adjustHeightToFit()


class Toolbar:
    def __init__(self, mw: aqt.AnkiQt, web: AnkiWebView) -> None:
        self.mw = mw
        self.web = web
        self.link_handlers: dict[str, Callable] = {
            "study": self._studyLinkHandler,
            "create": self._create_menu,
            "menu": self._app_menu,
        }
        self.web.requiresCol = False

    def draw(
        self,
        buf: str = "",
        web_context: Any | None = None,
        link_handler: Callable[[str], Any] | None = None,
    ) -> None:
        web_context = web_context or TopToolbar(self)
        link_handler = link_handler or self._linkHandler
        self.web.set_bridge_command(link_handler, web_context)
        body = self._body.format(
            gauge=_GAUGE_SVG,
            nav_content=self._nav_links(),
            actions_content=self._actions(),
            left_tray_content=self._left_tray_content(),
            right_tray_content=self._right_tray_content(),
        )
        self.web.stdHtml(
            body,
            css=["css/toolbar.css"],
            js=["js/vendor/jquery.min.js", "js/toolbar.js"],
            context=web_context,
        )
        self.web.adjustHeightToFit()

    def redraw(self) -> None:
        self.set_sync_active(self.mw.media_syncer.is_syncing())
        self.update_sync_status()
        gui_hooks.top_toolbar_did_redraw(self)

    # Available links
    ######################################################################

    def create_link(
        self,
        cmd: str,
        label: str,
        func: Callable,
        tip: str | None = None,
        id: str | None = None,
        active: bool = False,
    ) -> str:
        """Generates HTML link element and registers link handler

        Arguments:
            cmd {str} -- Command name used for the JS → Python bridge
            label {str} -- Display label of the link
            func {Callable} -- Callable to be called on clicking the link

        Keyword Arguments:
            tip {Optional[str]} -- Optional tooltip text to show on hovering
                                   over the link (default: {None})
            id: {Optional[str]} -- Optional id attribute to supply the link with
                                   (default: {None})
            active {bool} -- Mark the nav item as the current section.

        Returns:
            str -- HTML link element
        """

        self.link_handlers[cmd] = func

        title_attr = f'title="{tip}"' if tip else ""
        id_attr = f'id="{id}"' if id else ""
        klass = "hitem active" if active else "hitem"

        return (
            f"""<a class="{klass}" tabindex="-1" aria-label="{label}" """
            f"""{title_attr} {id_attr} href=# onclick="return pycmd('{cmd}')">"""
            f"""{label}</a>"""
        )

    def _nav_links(self) -> str:
        # The main window is always the deck flow, so Decks is the live section;
        # Browse and Stats open their own windows over it.
        links = [
            self.create_link(
                "decks",
                tr.actions_decks(),
                self._deckLinkHandler,
                tip=tr.actions_shortcut_key(val="D"),
                id="decks",
                active=True,
            ),
            self.create_link(
                "browse",
                tr.qt_misc_browse(),
                self._browseLinkHandler,
                tip=tr.actions_shortcut_key(val="B"),
                id="browse",
            ),
            self.create_link(
                "stats",
                tr.qt_misc_stats(),
                self._statsLinkHandler,
                tip=tr.actions_shortcut_key(val="T"),
                id="stats",
            ),
        ]

        gui_hooks.top_toolbar_did_init_links(links, self)

        return "\n".join(links)

    def _actions(self) -> str:
        """The right cluster: Create, Sync, and the overflow menu."""
        create = (
            '<button class="create" tabindex="-1" title="Create" aria-label="Create" '
            "onclick=\"return pycmd('create')\">{plus}<span>Create</span></button>"
        ).format(plus=_PLUS_SVG)
        menu = (
            '<button id="appmenu" class="iconbtn" tabindex="-1" aria-label="Menu" '
            'title="Menu" onclick="return pycmd(\'menu\')">{icon}</button>'
        ).format(icon=_MENU_SVG)
        return create + self._create_sync_link() + menu

    # Add-ons
    ######################################################################

    def _left_tray_content(self) -> str:
        left_tray_content: list[str] = []
        gui_hooks.top_toolbar_will_set_left_tray_content(left_tray_content, self)
        return self._process_tray_content(left_tray_content)

    def _right_tray_content(self) -> str:
        right_tray_content: list[str] = []
        gui_hooks.top_toolbar_will_set_right_tray_content(right_tray_content, self)
        return self._process_tray_content(right_tray_content)

    def _process_tray_content(self, content: list[str]) -> str:
        return "\n".join(f"""<div class="tray-item">{item}</div>""" for item in content)

    # Sync
    ######################################################################

    def _create_sync_link(self) -> str:
        name = tr.qt_misc_sync()
        title = tr.actions_shortcut_key(val="Y")
        label = "sync"
        self.link_handlers[label] = self._syncLinkHandler

        return f"""
<a class="iconbtn sync" tabindex="-1" aria-label="{name}" title="{name} ({title})" id="{label}" href=# onclick="return pycmd('{label}')"
>{_SYNC_SVG}<img id=sync-spinner src='/_anki/imgs/refresh.svg'>
</a>"""

    def set_sync_active(self, active: bool) -> None:
        method = "add" if active else "remove"
        self.web.eval(
            f"document.getElementById('sync-spinner').classList.{method}('spin')"
        )

    def set_sync_status(self, status: SyncStatus) -> None:
        self.web.eval(f"updateSyncColor({status.required})")

    def update_sync_status(self) -> None:
        get_sync_status(self.mw, self.mw.toolbar.set_sync_status)

    # Link handling
    ######################################################################

    def _linkHandler(self, link: str) -> bool:
        if link in self.link_handlers:
            self.link_handlers[link]()
        return False

    def _deckLinkHandler(self) -> None:
        self.mw.moveToState("deckBrowser")

    def _studyLinkHandler(self) -> None:
        # if overview already shown, switch to review
        if self.mw.state == "overview":
            self.mw.col.startTimebox()
            self.mw.moveToState("review")
        else:
            self.mw.onOverview()

    def _addLinkHandler(self) -> None:
        self.mw.onAddCard()

    def _browseLinkHandler(self) -> None:
        self.mw.onBrowse()

    def _statsLinkHandler(self) -> None:
        self.mw.onStats()

    def _syncLinkHandler(self) -> None:
        self.mw.on_sync_button_clicked()

    # Create + overflow menus
    ######################################################################

    def _create_menu(self) -> None:
        """The + Create menu: the app's single create surface."""
        m = QMenu(self.mw)
        # QMenu.addAction is typed Optional but never returns None here.
        for label, handler in (
            (tr.actions_add(), self._addLinkHandler),
            (tr.decks_create_deck(), self._on_create_deck),
            (tr.decks_import_file(), lambda: self.mw.onImport()),
        ):
            action = m.addAction(label)
            assert action is not None
            qconnect(action.triggered, handler)
        m.addSeparator()
        shared = m.addAction(tr.decks_get_shared())
        assert shared is not None
        qconnect(shared.triggered, lambda: openLink(f"{aqt.appShared}decks/"))
        m.popup(QCursor.pos())

    def _on_create_deck(self) -> None:
        from aqt.operations.deck import add_deck_dialog

        if op := add_deck_dialog(
            parent=self.mw, default_text=self.mw.col.decks.current()["name"]
        ):
            op.run_in_background()

    def _app_menu(self) -> None:
        """The overflow menu. Mirrors the native menu bar so every action stays
        reachable in-window; reuses the live QActions (dynamic labels, shortcuts,
        and any add-on-added items come along for free)."""
        f = self.mw.form
        m = QMenu(self.mw)
        # Undo / Redo lead.
        m.addActions(f.menuEdit.actions())
        m.addSeparator()
        for src in (
            f.menuCol,
            f.menuTools,
            f.menuqt_accel_view,
            f.menuHelp,
        ):
            sub = m.addMenu(src.title())
            assert sub is not None
            sub.addActions(src.actions())
        m.popup(QCursor.pos())

    # HTML & CSS
    ######################################################################

    _body = """
<div class="appbar">
  <a class="brand" tabindex="-1" href=# aria-label="Speedrun" onclick="return pycmd('decks')">
    <span class="brand-mark">{gauge}</span><span class="brand-word">speedrun</span>
  </a>
  <div class="left-tray">{left_tray_content}</div>
  <nav class="toolbar" aria-label="Primary">{nav_content}</nav>
  <div class="appbar-spacer"></div>
  <div class="actions">{actions_content}</div>
  <div class="right-tray">{right_tray_content}</div>
</div>
"""


# Bottom bar
######################################################################


class BottomBar(Toolbar):
    _centerBody = """
<center id=outer><table width=100%% id=header><tr><td align=center>
%s</td></tr></table></center>
"""

    def draw(
        self,
        buf: str = "",
        web_context: Any | None = None,
        link_handler: Callable[[str], Any] | None = None,
    ) -> None:
        # note: some screens may override this
        web_context = web_context or BottomToolbar(self)
        link_handler = link_handler or self._linkHandler
        self.web.set_bridge_command(link_handler, web_context)
        self.web.stdHtml(
            self._centerBody % buf,
            css=["css/toolbar.css", "css/toolbar-bottom.css"],
            context=web_context,
        )
        self.web.adjustHeightToFit()
