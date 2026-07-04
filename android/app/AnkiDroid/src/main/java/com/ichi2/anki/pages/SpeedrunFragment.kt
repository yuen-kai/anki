// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: Copyright (c) 2026 Ankitects Pty Ltd and contributors

package com.ichi2.anki.pages

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.net.Uri
import android.os.Bundle
import android.view.View
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebChromeClient.FileChooserParams
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import androidx.activity.addCallback
import androidx.activity.result.contract.ActivityResultContracts
import com.ichi2.anki.R
import com.ichi2.anki.SingleFragmentActivity
import com.ichi2.anki.ViewerResourceHandler
import com.ichi2.anki.utils.openUrl
import timber.log.Timber

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

    // The pending WebView callback from an `<input type="file">`, fulfilled (or
    // cancelled with null) by the document-picker result. The AI-import screen
    // (speedrun-import) uses a file input to add source material.
    private var fileChooserCallback: ValueCallback<Array<Uri>>? = null

    private val fileChooserLauncher =
        registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            val callback = fileChooserCallback ?: return@registerForActivityResult
            fileChooserCallback = null
            callback.onReceiveValue(FileChooserParams.parseResult(result.resultCode, result.data))
        }

    override fun onCreateWebViewClient(savedInstanceState: Bundle?): PageWebViewClient = SpeedrunWebViewClient(requireContext())

    override fun onCreateWebChromeClient(savedInstanceState: Bundle?): WebChromeClient = SpeedrunChromeClient()

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

    /**
     * Routes the shell's `<input type="file">` (the AI-import file picker) to the
     * system document picker, then hands the chosen URIs back to the WebView.
     */
    private inner class SpeedrunChromeClient : PageChromeClient() {
        override fun onShowFileChooser(
            webView: WebView?,
            filePathCallback: ValueCallback<Array<Uri>>?,
            fileChooserParams: FileChooserParams?,
        ): Boolean {
            if (filePathCallback == null || fileChooserParams == null) return false
            // Cancel any prior pending request so its input isn't left hanging.
            fileChooserCallback?.onReceiveValue(null)
            fileChooserCallback = filePathCallback
            return try {
                fileChooserLauncher.launch(fileChooserParams.createIntent())
                true
            } catch (_: ActivityNotFoundException) {
                Timber.w("No activity to handle file chooser")
                fileChooserCallback = null
                false
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
private class SpeedrunWebViewClient(
    context: Context,
) : PageWebViewClient() {
    // Serve collection media the same way the classic reviewer does, so a
    // Speedrun figure referenced as `<img src="/x.png">` resolves from the
    // media dir. The base client only serves the bundled SvelteKit assets.
    private val resourceHandler = ViewerResourceHandler(context)

    override fun shouldInterceptRequest(
        view: WebView,
        request: WebResourceRequest,
    ): WebResourceResponse? =
        super.shouldInterceptRequest(view, request)
            ?: resourceHandler.shouldInterceptRequest(request)

    override fun onPageStarted(
        view: WebView?,
        url: String?,
        favicon: Bitmap?,
    ) {
        super.onPageStarted(view, url, favicon)
        view?.evaluateJavascript("window.speedrunPlatform = 'mobile';", null)
    }

    override fun shouldOverrideUrlLoading(
        view: WebView?,
        request: WebResourceRequest?,
    ): Boolean {
        // The shell is served from the local backend and routes between its own
        // screens client-side, so a real navigation to a remote http(s) page
        // (e.g. the account screen's "Create an AnkiWeb account" link) is an
        // external link: open it in the system browser instead of loading it
        // over the single-page shell.
        val url = request?.url ?: return super.shouldOverrideUrlLoading(view, request)
        val isExternalLink =
            url.scheme in listOf("http", "https") && url.host != AnkiServer.LOCALHOST
        if (isExternalLink) {
            view?.context?.openUrl(url)
            return true
        }
        return super.shouldOverrideUrlLoading(view, request)
    }
}
