// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: Copyright (c) 2026 Ankitects Pty Ltd and contributors

package com.ichi2.anki.pages

import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.os.Bundle
import android.view.View
import android.webkit.WebView
import androidx.activity.addCallback
import com.ichi2.anki.R
import com.ichi2.anki.SingleFragmentActivity

/**
 * The app's primary UI: the Speedrun SvelteKit shell, hosted full-window with no
 * top app bar. It boots at the `speedrun-decks` home; every other Speedrun
 * screen (deck overview, hierarchy authoring, dashboard, study session) is
 * reached by client-side SvelteKit routing within this one WebView, so the
 * shell replaces AnkiDroid's stock deck list as the launch destination.
 *
 * The screens drive the shared Rust engine over the same local backend bridge
 * the desktop uses (see [PostRequestHandler.collectionMethods] and the Speedrun
 * RPCs on SchedulerService), so there is no per-platform study/authoring logic.
 */
class SpeedrunFragment : PageFragment(R.layout.fragment_speedrun) {
    override val pagePath: String
        get() = "speedrun-decks"

    override fun onCreateWebViewClient(savedInstanceState: Bundle?): PageWebViewClient = SpeedrunWebViewClient()

    override fun onViewCreated(
        view: View,
        savedInstanceState: Bundle?,
    ) {
        super.onViewCreated(view, savedInstanceState)
        // The shell navigates client-side (history.pushState), so the system
        // back button should walk the WebView history first, and only exit the
        // activity once we are back at the home screen.
        requireActivity().onBackPressedDispatcher.addCallback(viewLifecycleOwner) {
            if (webViewLayout.canGoBack()) {
                webViewLayout.goBack()
            } else {
                isEnabled = false
                requireActivity().onBackPressedDispatcher.onBackPressed()
            }
        }
    }

    companion object {
        fun getIntent(context: Context): Intent =
            SingleFragmentActivity
                .getIntent(context, SpeedrunFragment::class)
    }
}

/**
 * Tags the WebView as the mobile Speedrun shell so the shared SvelteKit code
 * routes cross-screen navigation client-side (instead of the desktop's Qt
 * `moveToState` RPCs). Read by `isMobileShell()` in the frontend. Set on every
 * page start; it persists on `window` for the SPA's lifetime and is read at
 * navigation time, well after load.
 */
private class SpeedrunWebViewClient : PageWebViewClient() {
    override fun onPageStarted(
        view: WebView?,
        url: String?,
        favicon: Bitmap?,
    ) {
        super.onPageStarted(view, url, favicon)
        view?.evaluateJavascript("window.speedrunPlatform = 'mobile';", null)
    }
}
