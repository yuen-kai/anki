// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: Copyright (c) 2026 Ankitects Pty Ltd and contributors

package com.ichi2.anki.pages

import android.content.Context
import android.content.Intent
import androidx.core.os.bundleOf
import com.ichi2.anki.SingleFragmentActivity
import com.ichi2.anki.libanki.DeckId

/**
 * Hosts the bespoke Speedrun study screen (the SvelteKit `speedrun-review`
 * route) in a WebView. The screen drives the shared Rust engine over the same
 * local backend bridge the desktop uses (see [PostRequestHandler.collectionMethods]
 * and the speedrun RPCs on SchedulerService), so there is no per-platform
 * study logic.
 */
class SpeedrunReviewFragment : PageFragment() {
    override val pagePath: String
        get() = "speedrun-review/${requireArguments().getLong(ARG_DECK_ID)}"

    companion object {
        private const val ARG_DECK_ID = "deckId"

        fun getIntent(
            context: Context,
            deckId: DeckId,
        ): Intent =
            SingleFragmentActivity.getIntent(
                context,
                SpeedrunReviewFragment::class,
                bundleOf(ARG_DECK_ID to deckId),
            )
    }
}
