/*
 * Copyright (c) 2022 Ankitects Pty Ltd <http://apps.ankiweb.net>
 *
 * This program is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation; either version 3 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 * PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program.  If not, see <http://www.gnu.org/licenses/>.
 */

package com.ichi2.anki

import androidx.annotation.StringRes
import androidx.appcompat.app.AlertDialog
import androidx.fragment.app.FragmentActivity
import anki.collection.Progress
import anki.sync.SyncAuth
import anki.sync.SyncCollectionResponse
import anki.sync.syncAuth
import com.google.android.material.snackbar.Snackbar
import com.ichi2.anki.CollectionManager.TR
import com.ichi2.anki.CollectionManager.withCol
import com.ichi2.anki.common.time.TimeManager
import com.ichi2.anki.dialogs.SyncErrorDialog
import com.ichi2.anki.observability.ChangeManager.notifySubscribersAllValuesChanged
import com.ichi2.anki.settings.Prefs
import com.ichi2.anki.settings.enums.ShouldFetchMedia
import com.ichi2.anki.snackbar.showSnackbar
import com.ichi2.anki.ui.internationalization.sentenceCase
import com.ichi2.anki.worker.SyncMediaWorker
import com.ichi2.preferences.VersatileTextWithASwitchPreference
import com.ichi2.utils.NetworkUtils
import com.ichi2.utils.dismissSafely
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import net.ankiweb.rsdroid.Backend
import net.ankiweb.rsdroid.exceptions.BackendInterruptedException
import net.ankiweb.rsdroid.exceptions.BackendSyncException
import timber.log.Timber
import kotlin.coroutines.resume

object SyncPreferences {
    const val CURRENT_SYNC_URI = "currentSyncUri"
    const val CUSTOM_SYNC_URI = "syncBaseUrl"
    const val CUSTOM_SYNC_ENABLED = CUSTOM_SYNC_URI + VersatileTextWithASwitchPreference.SWITCH_SUFFIX
}

enum class ConflictResolution {
    FULL_DOWNLOAD,
    FULL_UPLOAD,
}

fun syncAuth(): SyncAuth? {
    // Grab custom sync certificate from preferences (default is the empty string) and set it in CollectionManager
    val currentSyncCertificate = Prefs.customSyncCertificate ?: ""
    CollectionManager.updateCustomCertificate(currentSyncCertificate)

    val resolvedEndpoint = getEndpoint()
    return Prefs.hkey?.let {
        syncAuth {
            this.hkey = it
            if (resolvedEndpoint != null) {
                this.endpoint = resolvedEndpoint
            }
            this.ioTimeoutSecs = Prefs.networkTimeoutSecs
        }
    }
}

fun getEndpoint(): String? {
    val currentEndpoint = Prefs.currentSyncUri?.ifEmpty { null }
    val customEndpoint =
        if (Prefs.isCustomSyncEnabled) {
            Prefs.customSyncUri
        } else {
            null
        }
    return currentEndpoint ?: customEndpoint
}

/**
 * Whether the user has a sync account.
 * Returning true does not guarantee that the user actually synced recently,
 * or even that the ankiweb account is still valid.
 */
fun isLoggedIn(): Boolean = !Prefs.hkey.isNullOrEmpty()

fun millisecondsSinceLastSync() = TimeManager.time.intTimeMS() - Prefs.lastSyncTime

fun DeckPicker.handleNewSync(
    conflict: ConflictResolution?,
    syncMedia: Boolean,
) {
    val auth = syncAuth() ?: return
    val deckPicker = this
    launchCatchingTask {
        try {
            try {
                when (conflict) {
                    ConflictResolution.FULL_DOWNLOAD -> handleDownload(deckPicker, auth, deckPicker.mediaUsnOnConflict)
                    ConflictResolution.FULL_UPLOAD -> handleUpload(deckPicker, auth, deckPicker.mediaUsnOnConflict)
                    null -> {
                        handleNormalSync(deckPicker, auth, syncMedia)
                    }
                }
            } catch (exc: BackendSyncException.BackendSyncAuthFailedException) {
                // auth failed; log out
                updateLogin("", "")
                throw exc
            }
            withCol { notetypes.clearCache() }
            notifySubscribersAllValuesChanged(deckPicker)
            refreshState()
        } finally {
            // Always update last sync time to prevent infinite retry loops
            // when sync fails (e.g., collection too large). See issue #19776
            setLastSyncTimeToNow()
        }
    }
}

fun updateLogin(
    username: String,
    hkey: String,
) {
    Prefs.username = username
    Prefs.hkey = hkey
}

fun cancelSync(backend: Backend) {
    backend.setWantsAbort()
    backend.abortSync()
}

private suspend fun handleNormalSync(
    deckPicker: DeckPicker,
    auth: SyncAuth,
    syncMedia: Boolean,
) {
    Timber.i("Sync: Normal collection sync")
    var auth2 = auth
    val output =
        deckPicker.withProgress(
            extractProgress = {
                if (progress.hasNormalSync()) {
                    text = progress.normalSync.run { "$added\n$removed" }
                }
            },
            onCancel = ::cancelSync,
            manualCancelButton = R.string.dialog_cancel,
        ) {
            withCol {
                syncCollection(auth2, syncMedia = false) // media is synced by SyncMediaWorker
            }
        }

    if (output.hasNewEndpoint() && output.newEndpoint.isNotEmpty()) {
        Timber.i("sync endpoint updated")
        Prefs.currentSyncUri = output.newEndpoint
        auth2 =
            syncAuth {
                this.hkey = auth.hkey
                endpoint = output.newEndpoint
            }
    }
    val mediaUsn =
        if (syncMedia) {
            output.serverMediaUsn
        } else {
            null
        }

    Timber.i("sync result: ${output.required}")
    when (output.required) {
        // a successful sync returns this value
        SyncCollectionResponse.ChangesRequired.NO_CHANGES -> {
            // scheduler version may have changed
            withCol { _loadScheduler() }
            val message = if (syncMedia) R.string.col_synced_media_in_background else R.string.sync_database_acknowledge
            deckPicker.showSyncLogMessage(message, output.serverMessage)
            deckPicker.refreshState()
            if (syncMedia) {
                SyncMediaWorker.start(deckPicker, auth2)
            }
        }

        SyncCollectionResponse.ChangesRequired.FULL_DOWNLOAD -> {
            handleDownload(deckPicker, auth2, mediaUsn)
        }

        SyncCollectionResponse.ChangesRequired.FULL_UPLOAD -> {
            handleUpload(deckPicker, auth2, mediaUsn)
        }

        SyncCollectionResponse.ChangesRequired.FULL_SYNC -> {
            deckPicker.mediaUsnOnConflict = mediaUsn
            deckPicker.showSyncErrorDialog(SyncErrorDialog.Type.DIALOG_SYNC_CONFLICT_RESOLUTION)
        }

        SyncCollectionResponse.ChangesRequired.NORMAL_SYNC,
        SyncCollectionResponse.ChangesRequired.UNRECOGNIZED,
        null,
        -> {
            TODO("should never happen")
        }
    }
}

private fun fullDownloadProgress(title: String): ProgressContext.() -> Unit =
    {
        fun Progress.FullSync.toAmount() = ProgressContext.Amount(transferred.toLong(), total.toLong())

        text = title
        if (progress.hasFullSync() && progress.fullSync.total > 0) {
            amount = progress.fullSync.toAmount()
        }
    }

private suspend fun handleDownload(
    deckPicker: DeckPicker,
    auth: SyncAuth,
    mediaUsn: Int?,
) {
    Timber.i("Sync: Full collection download requested")
    deckPicker.withProgress(
        progressContext = ProgressContext.ofBytes(context = deckPicker).copy(separator = "\n"),
        extractProgress = fullDownloadProgress(TR.syncDownloadingFromAnkiweb()),
        onCancel = ::cancelSync,
        manualCancelButton = R.string.dialog_cancel,
    ) {
        withCol {
            try {
                createBackup(
                    BackupManager.getBackupDirectoryFromCollection(colDb),
                    force = true,
                    waitForCompletion = true,
                )
                close(downgrade = false, forFullSync = true)
                fullUploadOrDownload(auth, upload = false, serverUsn = mediaUsn)
            } finally {
                reopen(afterFullSync = true)
            }
        }
        deckPicker.refreshState()
        if (mediaUsn != null) {
            SyncMediaWorker.start(deckPicker, auth)
        }
    }

    Timber.i("Full Download Completed")
    deckPicker.showSyncLogMessage(R.string.backup_one_way_sync_from_server, "")
}

private suspend fun handleUpload(
    deckPicker: DeckPicker,
    auth: SyncAuth,
    mediaUsn: Int?,
) {
    Timber.i("Sync: Full collection upload requested")
    deckPicker.withProgress(
        progressContext = ProgressContext.ofBytes(context = deckPicker).copy(separator = "\n"),
        extractProgress = fullDownloadProgress(TR.syncUploadingToAnkiweb()),
        onCancel = ::cancelSync,
        manualCancelButton = R.string.dialog_cancel,
    ) {
        withCol {
            close(downgrade = false, forFullSync = true)
            try {
                fullUploadOrDownload(auth, upload = true, serverUsn = mediaUsn)
            } finally {
                reopen(afterFullSync = true)
            }
        }
        deckPicker.refreshState()
        if (mediaUsn != null) {
            SyncMediaWorker.start(deckPicker, auth)
        }
    }
    Timber.i("Full Upload Completed")
    deckPicker.showSyncLogMessage(R.string.sync_log_uploading_message, "")
}

// --- Speedrun shell sync ----------------------------------------------------
//
// The Speedrun mobile shell replaces DeckPicker as the phone's home, so its
// account screen's "Sync now" can't reuse DeckPicker.handleNewSync. It used to
// fire the background SyncWorker, which deliberately SKIPS one-way syncs
// (FULL_DOWNLOAD/FULL_UPLOAD/FULL_SYNC) because they need user input. That
// silently no-op'd the phone's first sync against an AnkiWeb account already
// holding the desktop collection, so the phone never synced. This mirrors the
// desktop's interactive speedrun_sync_now: run the native sync from the shell's
// host activity and resolve a required one-way sync with a Download/Upload
// choice. A normal sync still merges reviews and deck/card edits both ways via
// the shared Rust engine.

/** Result of a Speedrun sync: either it completed, or a one-way full sync is
 * needed and was left for the user to resolve. */
enum class SpeedrunSyncOutcome {
    SYNCED,
    NEEDS_RESOLUTION,
}

/**
 * Run a collection sync for the Speedrun shell from its host [FragmentActivity].
 *
 * When [interactive] is true (the "Sync now" button) it shows a progress dialog
 * and resolves a required one-way sync with a Download/Upload dialog. When false
 * (the periodic auto-sync) it runs silently and only performs a normal two-way
 * merge; a required one-way full sync is skipped and reported as
 * [SpeedrunSyncOutcome.NEEDS_RESOLUTION] so the UI can prompt later.
 *
 * Throws [BackendSyncException.BackendSyncAuthFailedException] on bad credentials
 * so the caller can sign the user out.
 */
suspend fun FragmentActivity.speedrunSyncCollection(
    auth: SyncAuth,
    syncMedia: Boolean,
    interactive: Boolean = true,
): SpeedrunSyncOutcome {
    var auth2 = auth
    val runNormalSync: suspend () -> SyncCollectionResponse = {
        withCol { syncCollection(auth2, syncMedia = false) } // media synced separately below
    }
    val output =
        if (interactive) {
            withProgress(
                extractProgress = {
                    if (progress.hasNormalSync()) {
                        text = progress.normalSync.run { "$added\n$removed" }
                    }
                },
                onCancel = ::cancelSync,
                manualCancelButton = R.string.dialog_cancel,
            ) {
                runNormalSync()
            }
        } else {
            runNormalSync()
        }

    if (output.hasNewEndpoint() && output.newEndpoint.isNotEmpty()) {
        Timber.i("speedrun sync endpoint updated")
        Prefs.currentSyncUri = output.newEndpoint
        auth2 =
            syncAuth {
                this.hkey = auth.hkey
                endpoint = output.newEndpoint
            }
    }
    val mediaUsn = if (syncMedia) output.serverMediaUsn else null

    Timber.i("speedrun sync required: %s (interactive=%b)", output.required, interactive)
    val outcome =
        when (output.required) {
            // a successful (merging) sync returns this value
            SyncCollectionResponse.ChangesRequired.NO_CHANGES -> {
                withCol { _loadScheduler() } // scheduler version may have changed
                if (syncMedia) SyncMediaWorker.start(this, auth2)
                SpeedrunSyncOutcome.SYNCED
            }
            // A one-way sync needs user input, so only do it in the interactive
            // flow; auto-sync leaves it for the manual "Sync now" button.
            SyncCollectionResponse.ChangesRequired.FULL_DOWNLOAD ->
                if (interactive) {
                    speedrunFullSync(auth2, mediaUsn, upload = false)
                    SpeedrunSyncOutcome.SYNCED
                } else {
                    SpeedrunSyncOutcome.NEEDS_RESOLUTION
                }
            SyncCollectionResponse.ChangesRequired.FULL_UPLOAD ->
                if (interactive) {
                    speedrunFullSync(auth2, mediaUsn, upload = true)
                    SpeedrunSyncOutcome.SYNCED
                } else {
                    SpeedrunSyncOutcome.NEEDS_RESOLUTION
                }
            SyncCollectionResponse.ChangesRequired.FULL_SYNC ->
                if (interactive) {
                    when (speedrunResolveSyncConflict()) {
                        ConflictResolution.FULL_UPLOAD -> {
                            speedrunFullSync(auth2, mediaUsn, upload = true)
                            SpeedrunSyncOutcome.SYNCED
                        }
                        ConflictResolution.FULL_DOWNLOAD -> {
                            speedrunFullSync(auth2, mediaUsn, upload = false)
                            SpeedrunSyncOutcome.SYNCED
                        }
                        null -> SpeedrunSyncOutcome.NEEDS_RESOLUTION
                    }
                } else {
                    SpeedrunSyncOutcome.NEEDS_RESOLUTION
                }
            SyncCollectionResponse.ChangesRequired.NORMAL_SYNC,
            SyncCollectionResponse.ChangesRequired.UNRECOGNIZED,
            null,
            -> SpeedrunSyncOutcome.SYNCED
        }
    if (outcome == SpeedrunSyncOutcome.SYNCED) setLastSyncTimeToNow()
    return outcome
}

/** Full one-way sync for the Speedrun shell, mirroring [handleDownload]/[handleUpload]. */
private suspend fun FragmentActivity.speedrunFullSync(
    auth: SyncAuth,
    mediaUsn: Int?,
    upload: Boolean,
) {
    val title = if (upload) TR.syncUploadingToAnkiweb() else TR.syncDownloadingFromAnkiweb()
    withProgress(
        progressContext = ProgressContext.ofBytes(context = this).copy(separator = "\n"),
        extractProgress = fullDownloadProgress(title),
        onCancel = ::cancelSync,
        manualCancelButton = R.string.dialog_cancel,
    ) {
        withCol {
            if (!upload) {
                // A download overwrites the local collection, so back it up first.
                createBackup(
                    BackupManager.getBackupDirectoryFromCollection(colDb),
                    force = true,
                    waitForCompletion = true,
                )
            }
            close(downgrade = false, forFullSync = true)
            try {
                fullUploadOrDownload(auth, upload = upload, serverUsn = mediaUsn)
            } finally {
                reopen(afterFullSync = true)
            }
        }
        if (mediaUsn != null) SyncMediaWorker.start(this@speedrunFullSync, auth)
    }
    Timber.i("speedrun full %s completed", if (upload) "upload" else "download")
}

/**
 * Ask which side wins when the collections have diverged and a normal sync isn't
 * possible (FULL_SYNC): keep this device (upload) or take AnkiWeb (download).
 * Same choice AnkiDroid's DeckPicker offers via [SyncErrorDialog].
 */
private suspend fun FragmentActivity.speedrunResolveSyncConflict(): ConflictResolution? =
    suspendCancellableCoroutine { cont ->
        val dialog =
            AlertDialog
                .Builder(this)
                .setTitle(R.string.sync_conflict_title_new)
                .setMessage(R.string.sync_conflict_message_new)
                .setPositiveButton(R.string.sync_conflict_keep_local_new) { _, _ ->
                    if (cont.isActive) cont.resume(ConflictResolution.FULL_UPLOAD)
                }.setNegativeButton(R.string.sync_conflict_keep_remote_new) { _, _ ->
                    if (cont.isActive) cont.resume(ConflictResolution.FULL_DOWNLOAD)
                }.setNeutralButton(R.string.dialog_cancel) { _, _ ->
                    if (cont.isActive) cont.resume(null)
                }.setCancelable(false)
                .show()
        cont.invokeOnCancellation { dialog.dismissSafely() }
    }

fun cancelMediaSync(backend: Backend) {
    backend.setWantsAbort()
    backend.abortMediaSync()
}

/**
 * Whether media should be fetched on sync. Options from preferences are:
 * * Always
 * * Only if unmetered
 * * Never
 */
fun shouldFetchMedia(): Boolean {
    val shouldFetchMedia = Prefs.shouldFetchMedia
    return shouldFetchMedia == ShouldFetchMedia.ALWAYS ||
        (shouldFetchMedia == ShouldFetchMedia.ONLY_UNMETERED && !NetworkUtils.isActiveNetworkMetered())
}

suspend fun monitorMediaSync(deckPicker: DeckPicker) {
    val backend = CollectionManager.getBackend()
    val scope = CoroutineScope(Dispatchers.IO)
    var isAborted = false

    val dialog =
        withContext(Dispatchers.Main) {
            AlertDialog
                .Builder(deckPicker)
                .setTitle(with(deckPicker) { TR.sentenceCase.mediaSyncLog })
                .setMessage("")
                .setPositiveButton(R.string.dialog_continue) { _, _ ->
                    scope.cancel()
                }.setNegativeButton(TR.syncAbortButton()) { _, _ ->
                    isAborted = true
                    cancelMediaSync(backend)
                }.show()
        }

    fun showMessage(msg: String) = deckPicker.showSnackbar(msg, Snackbar.LENGTH_SHORT)

    scope.launch {
        try {
            while (true) {
                // this will throw if the sync exited with an error
                val resp = backend.mediaSyncStatus()
                if (!resp.active) {
                    break
                }
                val text = resp.progress.run { "$added\n$removed\n$checked" }
                dialog.setMessage(text)
                delay(100)
            }
            showMessage(if (isAborted) TR.syncMediaAborted() else TR.syncMediaComplete())
        } catch (_: BackendInterruptedException) {
            showMessage(TR.syncMediaAborted())
        } catch (_: CancellationException) {
            // do nothing
        } catch (_: Exception) {
            showMessage(TR.syncMediaFailed())
        } finally {
            dialog.dismissSafely()
        }
    }
}

/**
 * Show a simple snackbar message or notification if the activity is not in foreground
 * @param messageResource String resource for message
 */
fun DeckPicker.showSyncLogMessage(
    @StringRes messageResource: Int,
    syncMessage: String?,
) {
    if (activityPaused) {
        val res = AnkiDroidApp.appResources
        showSimpleNotification(
            res.getString(R.string.app_name),
            res.getString(messageResource),
            Channel.SYNC,
        )
    } else {
        if (syncMessage.isNullOrEmpty()) {
            showSnackbar(messageResource)
        } else {
            val res = AnkiDroidApp.appResources
            showSimpleMessageDialog(title = res.getString(messageResource), message = syncMessage)
        }
    }
}

fun setLastSyncTimeToNow() {
    Prefs.lastSyncTime = TimeManager.time.intTimeMS()
}
