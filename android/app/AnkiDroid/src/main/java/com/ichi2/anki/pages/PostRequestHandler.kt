/*
 *  Copyright (c) 2023 Brayan Oliveira <brayandso.dev@gmail.com>
 *  Copyright (c) 2024 David Allison <davidallisongithub@gmail.com>
 *  Copyright (c) 2024 voczi <dev@voczi.com>
 *
 *  This program is free software; you can redistribute it and/or modify it under
 *  the terms of the GNU General Public License as published by the Free Software
 *  Foundation; either version 3 of the License, or (at your option) any later
 *  version.
 *
 *  This program is distributed in the hope that it will be useful, but WITHOUT ANY
 *  WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 *  PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along with
 *  this program.  If not, see <http://www.gnu.org/licenses/>.
 */
package com.ichi2.anki.pages

import android.app.Activity
import androidx.annotation.VisibleForTesting
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.lifecycleScope
import anki.collection.OpChanges
import com.google.protobuf.ByteString
import com.ichi2.anki.CollectionManager
import com.ichi2.anki.CollectionManager.withCol
import com.ichi2.anki.NoteEditorFragment
import com.ichi2.anki.SpeedrunSyncOutcome
import com.ichi2.anki.getEndpoint
import com.ichi2.anki.importAnkiPackageUndoable
import com.ichi2.anki.importCsvRaw
import com.ichi2.anki.isLoggedIn
import com.ichi2.anki.launchCatchingTask
import com.ichi2.anki.libanki.Collection
import com.ichi2.anki.libanki.completeTagRaw
import com.ichi2.anki.libanki.getCsvMetadataRaw
import com.ichi2.anki.libanki.getDeckConfigsForUpdateRaw
import com.ichi2.anki.libanki.getDeckNamesRaw
import com.ichi2.anki.libanki.getFieldNamesRaw
import com.ichi2.anki.libanki.getImportAnkiPackagePresetsRaw
import com.ichi2.anki.libanki.getNotetypeNamesRaw
import com.ichi2.anki.libanki.sched.computeFsrsParamsRaw
import com.ichi2.anki.libanki.sched.computeOptimalRetentionRaw
import com.ichi2.anki.libanki.sched.getMemoryScoreRaw
import com.ichi2.anki.libanki.sched.getPerformanceScoreRaw
import com.ichi2.anki.libanki.sched.getReadinessScoreRaw
import com.ichi2.anki.libanki.sched.getSpeedrunScoreBreakdownRaw
import com.ichi2.anki.libanki.sched.simulateFsrsReviewRaw
import com.ichi2.anki.libanki.sched.speedrunAiConfigRaw
import com.ichi2.anki.libanki.sched.speedrunAiImportRaw
import com.ichi2.anki.libanki.sched.speedrunAnswerCardRaw
import com.ichi2.anki.libanki.sched.speedrunDeleteDeckRaw
import com.ichi2.anki.libanki.sched.speedrunEnsureSeededRaw
import com.ichi2.anki.libanki.sched.speedrunGetHierarchyRaw
import com.ichi2.anki.libanki.sched.speedrunListDecksRaw
import com.ichi2.anki.libanki.sched.speedrunNextCardRaw
import com.ichi2.anki.libanki.sched.speedrunRecordLearnedRaw
import com.ichi2.anki.libanki.sched.speedrunSaveHierarchyRaw
import com.ichi2.anki.libanki.sched.speedrunStudyHierarchyRaw
import com.ichi2.anki.libanki.sched.speedrunStudyStateRaw
import com.ichi2.anki.libanki.sched.speedrunStudySummaryRaw
import com.ichi2.anki.libanki.stats.cardStatsRaw
import com.ichi2.anki.libanki.stats.getGraphPreferencesRaw
import com.ichi2.anki.libanki.stats.graphsRaw
import com.ichi2.anki.libanki.stats.setGraphPreferencesRaw
import com.ichi2.anki.observability.undoableOp
import com.ichi2.anki.searchInBrowser
import com.ichi2.anki.settings.Prefs
import com.ichi2.anki.speedrunSyncCollection
import com.ichi2.anki.syncAuth
import com.ichi2.anki.updateLogin
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import net.ankiweb.rsdroid.exceptions.BackendSyncException
import org.json.JSONObject
import timber.log.Timber
import anki.generic.Json as GenericJson

interface PostRequestHandler {
    suspend fun handlePostRequest(
        uri: PostRequestUri,
        bytes: ByteArray,
    ): ByteArray
}

@JvmInline
value class PostRequestUri(
    val uri: String,
) {
    val ankidroidMethodName: String?
        get() =
            if (uri.startsWith(AnkiServer.ANKIDROID_PREFIX)) {
                uri.substring(AnkiServer.ANKIDROID_PREFIX.length)
            } else {
                null
            }

    val backendMethodName: String?
        get() =
            if (uri.startsWith(AnkiServer.ANKI_PREFIX)) {
                uri.substring(AnkiServer.ANKI_PREFIX.length)
            } else {
                null
            }

    val jsApiMethodName: String?
        get() =
            if (uri.startsWith(AnkiServer.ANKIDROID_JS_PREFIX)) {
                uri.substring(AnkiServer.ANKIDROID_JS_PREFIX.length)
            } else {
                null
            }

    override fun toString() = uri
}

fun <ByteArray> backendIdentity(bytes: ByteArray): ByteArray = bytes

typealias CollectionBackendInterface = Collection.(bytes: ByteArray) -> ByteArray

@VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
val collectionMethods =
    hashMapOf<String, CollectionBackendInterface>(
        "i18nResources" to { bytes -> i18nResourcesRaw(bytes) },
        "getGraphPreferences" to { _ -> getGraphPreferencesRaw() },
        "setGraphPreferences" to { bytes -> setGraphPreferencesRaw(bytes) },
        "graphs" to { bytes -> graphsRaw(bytes) },
        "getNotetypeNames" to { bytes -> getNotetypeNamesRaw(bytes) },
        "getDeckNames" to { bytes -> getDeckNamesRaw(bytes) },
        "getCsvMetadata" to { bytes -> getCsvMetadataRaw(bytes) },
        "importDone" to { bytes -> backendIdentity(bytes) },
        "getImportAnkiPackagePresets" to { bytes -> getImportAnkiPackagePresetsRaw(bytes) },
        "completeTag" to { bytes -> completeTagRaw(bytes) },
        "getFieldNames" to { bytes -> getFieldNamesRaw(bytes) },
        "cardStats" to { bytes -> cardStatsRaw(bytes) },
        "getDeckConfigsForUpdate" to { bytes -> getDeckConfigsForUpdateRaw(bytes) },
        "computeOptimalRetention" to { bytes -> computeOptimalRetentionRaw(bytes) },
        "computeFsrsParams" to { bytes -> computeFsrsParamsRaw(bytes) },
        "evaluateParamsLegacy" to { bytes -> evaluateParamsLegacyRaw(bytes) },
        "simulateFsrsReview" to { bytes -> simulateFsrsReviewRaw(bytes) },
        "getImageForOcclusion" to { bytes -> getImageForOcclusionRaw(bytes) },
        "getImageOcclusionNote" to { bytes -> getImageOcclusionNoteRaw(bytes) },
        "setWantsAbort" to { bytes -> setWantsAbortRaw(bytes) },
        "getSchedulingStatesWithContext" to { bytes -> getSchedulingStatesWithContextRaw(bytes) },
        "setSchedulingStates" to { bytes -> setSchedulingStatesRaw(bytes) },
        "getChangeNotetypeInfo" to { bytes -> getChangeNotetypeInfoRaw(bytes) },
        "changeNotetype" to { bytes -> changeNotetypeRaw(bytes) },
        "importJsonString" to { bytes -> importJsonStringRaw(bytes) },
        "importJsonFile" to { bytes -> importJsonFileRaw(bytes) },
        "congratsInfo" to { bytes -> congratsInfoRaw(bytes) },
        "getImageOcclusionFields" to { bytes -> getImageOcclusionFieldsRaw(bytes) },
        "getIgnoredBeforeCount" to { bytes -> getIgnoredBeforeCountRaw(bytes) },
        "getRetentionWorkload" to { bytes -> getRetentionWorkloadRaw(bytes) },
        "simulateFsrsWorkload" to { bytes -> simulateFsrsWorkloadRaw(bytes) },
        // https://github.com/ankitects/anki/pull/4326 -> saveCustomColours should be no-op in mobile clients
        "saveCustomColours" to { bytes -> backendIdentity(bytes) },
        // The whole Speedrun shell (study, deck/authoring CRUD, honest scores)
        // runs on the shared Rust engine's SchedulerService RPCs, so the
        // AnkiDroid WebView drives the same backend as desktop
        // (anki/scheduler.proto).
        "speedrunStudyState" to { bytes -> speedrunStudyStateRaw(bytes) },
        "speedrunNextCard" to { bytes -> speedrunNextCardRaw(bytes) },
        "speedrunAnswerCard" to { bytes -> speedrunAnswerCardRaw(bytes) },
        "speedrunRecordLearned" to { bytes -> speedrunRecordLearnedRaw(bytes) },
        "speedrunStudyHierarchy" to { bytes -> speedrunStudyHierarchyRaw(bytes) },
        "speedrunListDecks" to { bytes -> speedrunListDecksRaw(bytes) },
        "speedrunGetHierarchy" to { bytes -> speedrunGetHierarchyRaw(bytes) },
        "speedrunSaveHierarchy" to { bytes -> speedrunSaveHierarchyRaw(bytes) },
        "speedrunDeleteDeck" to { bytes -> speedrunDeleteDeckRaw(bytes) },
        "speedrunStudySummary" to { bytes -> speedrunStudySummaryRaw(bytes) },
        // Preload the bundled demo + MCAT decks (idempotent); the mobile shell's
        // decks screen calls this on load so it has content out of the box.
        "speedrunEnsureSeeded" to { bytes -> speedrunEnsureSeededRaw(bytes) },
        // AI deck-import: the OpenAI call runs in the shared Rust engine, so the
        // API key stays server-side and the AnkiDroid webview drives the same
        // backend as desktop (anki/scheduler.proto).
        "speedrunAiConfig" to { bytes -> speedrunAiConfigRaw(bytes) },
        "speedrunAiImport" to { bytes -> speedrunAiImportRaw(bytes) },
        "getMemoryScore" to { bytes -> getMemoryScoreRaw(bytes) },
        "getPerformanceScore" to { bytes -> getPerformanceScoreRaw(bytes) },
        "getReadinessScore" to { bytes -> getReadinessScoreRaw(bytes) },
        "getSpeedrunScoreBreakdown" to { bytes -> getSpeedrunScoreBreakdownRaw(bytes) },
    )

suspend fun handleCollectionPostRequest(
    methodName: String,
    bytes: ByteArray,
): ByteArray? = collectionMethods[methodName]?.let { method -> withCol { method.invoke(this, bytes) } }

typealias UIBackendInterface = FragmentActivity.(bytes: ByteArray) -> Deferred<ByteArray>

@VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
val uiMethods =
    hashMapOf<String, UIBackendInterface>(
        "searchInBrowser" to { bytes -> lifecycleScope.async { searchInBrowser(bytes) } },
        "updateDeckConfigs" to { bytes -> lifecycleScope.async { updateDeckConfigsRaw(bytes) } },
        "latestProgress" to { bytes ->
            lifecycleScope.async {
                withContext(Dispatchers.IO) {
                    CollectionManager.getBackend().latestProgressRaw(bytes)
                }
            }
        },
        "i18nResources" to { bytes ->
            lifecycleScope.async {
                withContext(Dispatchers.IO) {
                    CollectionManager.getBackend().i18nResourcesRaw(bytes)
                }
            }
        },
        "importCsv" to { bytes -> lifecycleScope.async { importCsvRaw(bytes) } },
        "importAnkiPackage" to { bytes -> lifecycleScope.async { importAnkiPackageUndoable(bytes) } },
        "addImageOcclusionNote" to { bytes ->
            lifecycleScope.async {
                withCol { addImageOcclusionNoteRaw(bytes) }
            }
        },
        "updateImageOcclusionNote" to { bytes ->
            lifecycleScope.async {
                withCol { updateImageOcclusionNoteRaw(bytes) }
            }
        },
        "deckOptionsReady" to { bytes -> lifecycleScope.async { deckOptionsReady(bytes) } },
        "deckOptionsRequireClose" to { bytes -> lifecycleScope.async { deckOptionsRequireClose(bytes) } },
        // Speedrun cross-screen navigation (open deck, start study, show decks)
        // is client-side SvelteKit routing in the mobile shell, so there are no
        // navigation RPCs to handle here; the desktop keeps them as Qt actions.
        //
        // Speedrun accounts + sync (the speedrun-account screen). These mirror
        // the desktop handlers in qt/aqt/mediasrv.py: the page collects a
        // username + password, the host logs in to the self-hosted sync server
        // and stores the returned key like any other login. They exchange the
        // protobuf generic.Json wrapper via the FrontendService Speedrun* RPCs.
        "speedrunSyncLogin" to { bytes -> lifecycleScope.async { speedrunSyncLogin(bytes) } },
        "speedrunSyncNow" to { bytes -> lifecycleScope.async { speedrunSyncNow(bytes) } },
        "speedrunSignOut" to { _ -> lifecycleScope.async { speedrunSignOut() } },
        "speedrunSyncStatus" to { _ -> lifecycleScope.async { speedrunSyncStatus() } },
    )

// The account screen posts the protobuf generic.Json wrapper (an
// application/binary body whose `json` field holds the UTF-8 payload) via the
// FrontendService Speedrun* RPCs, exactly like the other Speedrun handlers and
// desktop's mediasrv `_speedrun_request` / `_speedrun_response`. Decode and
// encode that wrapper here so the same account lib.ts drives both hosts; the
// field names inside stay identical to mediasrv.py.
private fun parseJsonRequest(bytes: ByteArray): JSONObject {
    if (bytes.isEmpty()) return JSONObject()
    return try {
        val payload = GenericJson.parseFrom(bytes).json
        if (payload.isEmpty) JSONObject() else JSONObject(payload.toStringUtf8())
    } catch (_: Exception) {
        JSONObject()
    }
}

private fun jsonResponse(build: JSONObject.() -> Unit): ByteArray =
    GenericJson
        .newBuilder()
        .setJson(ByteString.copyFromUtf8(JSONObject().apply(build).toString()))
        .build()
        .toByteArray()

/**
 * Log in to the self-hosted sync server with a username and password, then store
 * the returned key like a normal login so the usual sync flow can use it.
 * Mirrors desktop's `speedrun_sync_login`.
 */
private suspend fun FragmentActivity.speedrunSyncLogin(bytes: ByteArray): ByteArray {
    val request = parseJsonRequest(bytes)
    val username = request.optString("username").trim()
    val password = request.optString("password")
    val endpoint = request.optString("endpoint").trim().ifEmpty { null }
    if (username.isEmpty() || password.isEmpty()) {
        return jsonResponse {
            put("ok", false)
            put("account", JSONObject.NULL)
            put("endpoint", JSONObject.NULL)
            put("message", "Enter a username and password.")
        }
    }
    return try {
        val auth = withCol { syncLogin(username, password, endpoint) }
        updateLogin(username, auth.hkey)
        if (endpoint != null) {
            Prefs.currentSyncUri = endpoint
        }
        jsonResponse {
            put("ok", true)
            put("account", username)
            put("endpoint", endpoint ?: JSONObject.NULL)
        }
    } catch (exc: Exception) {
        // do not log the error, it can contain PII (matches LoginViewModel)
        Timber.w("speedrun sync login failed")
        jsonResponse {
            put("ok", false)
            put("message", exc.localizedMessage ?: "login failed")
            put("account", JSONObject.NULL)
            put("endpoint", JSONObject.NULL)
        }
    }
}

/**
 * Run a collection sync against the stored credentials and wait for it to
 * finish, mirroring desktop's `speedrun_sync_now`. Uses the interactive sync
 * ([speedrunSyncCollection]) rather than the background SyncWorker, because the
 * worker skips one-way syncs and so the phone's first sync (a full download
 * from AnkiWeb) would silently do nothing.
 */
private suspend fun FragmentActivity.speedrunSyncNow(bytes: ByteArray): ByteArray {
    // interactive (the button) shows progress + a conflict dialog; the periodic
    // auto-sync passes interactive=false for a quiet, normal-only merge.
    val interactive = parseJsonRequest(bytes).optBoolean("interactive", true)
    val auth =
        syncAuth() ?: return jsonResponse {
            put("ok", false)
            put("status", "not-signed-in")
            put("message", "Not signed in.")
        }
    return try {
        val outcome = speedrunSyncCollection(auth, syncMedia = true, interactive = interactive)
        jsonResponse {
            when (outcome) {
                SpeedrunSyncOutcome.SYNCED -> {
                    put("ok", true)
                    put("status", "synced")
                    put("message", "Sync complete.")
                }
                SpeedrunSyncOutcome.NEEDS_RESOLUTION -> {
                    put("ok", true)
                    put("status", "conflict")
                }
            }
        }
    } catch (cancellation: CancellationException) {
        throw cancellation
    } catch (authFailed: BackendSyncException.BackendSyncAuthFailedException) {
        // credentials no longer valid: drop them so the screen shows signed-out
        updateLogin("", "")
        jsonResponse {
            put("ok", false)
            put("status", "error")
            put("message", authFailed.localizedMessage ?: "Sign in again.")
        }
    } catch (exc: Exception) {
        // do not log the message, it can contain PII (matches LoginViewModel)
        Timber.w("speedrun sync failed")
        jsonResponse {
            put("ok", false)
            put("status", "error")
            put("message", exc.localizedMessage ?: "Sync failed.")
        }
    }
}

/** Drop the stored sync credentials. Mirrors desktop's `speedrun_sign_out`. */
private fun speedrunSignOut(): ByteArray {
    updateLogin("", "")
    return jsonResponse { put("ok", true) }
}

/** Report the host's sync status. Mirrors desktop's `speedrun_sync_status`. */
private fun speedrunSyncStatus(): ByteArray {
    val loggedIn = isLoggedIn()
    return jsonResponse {
        put("loggedIn", loggedIn)
        put("account", if (loggedIn) (Prefs.username ?: JSONObject.NULL) else JSONObject.NULL)
        put("endpoint", getEndpoint() ?: JSONObject.NULL)
        put("hostAvailable", true)
    }
}

sealed class UiPostRequestResponse {
    /** The requested method was not a valid UI POST request */
    data object UnknownMethod : UiPostRequestResponse()

    /**
     * A valid method could not be executed
     *
     * For example: if the calling fragment was not attached to an activity
     */
    data object Ignored : UiPostRequestResponse()

    /**
     * The request was handled by the backend (success/failure)
     */
    data class Handled(
        val data: ByteArray,
    ) : UiPostRequestResponse() {
        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as Handled

            return data.contentEquals(other.data)
        }

        override fun hashCode(): Int = data.contentHashCode()
    }
}

suspend fun FragmentActivity?.handleUiPostRequest(
    methodName: String,
    bytes: ByteArray,
): UiPostRequestResponse {
    // an unknown method may be valid for another request handler
    val uiMethod = uiMethods[methodName] ?: return UiPostRequestResponse.UnknownMethod

    // a resolved but ignored method should not be retried
    if (this == null) {
        Timber.w("ignored UI request '%s' - activity == null", methodName)
        return UiPostRequestResponse.Ignored
    }

    val data = uiMethod.invoke(this, bytes).await()
    when (methodName) {
        "addImageOcclusionNote" -> {
            undoableOp { OpChanges.parseFrom(data) }
            launchCatchingTask {
                // Allow time for toast message to appear before closing editor
                delay(1000)
                setResult(Activity.RESULT_OK)
                finish()
            }
        }

        "updateImageOcclusionNote" -> {
            undoableOp { OpChanges.parseFrom(data) }
            launchCatchingTask {
                // Allow time for toast message to appear before closing editor
                delay(1000)
                setResult(NoteEditorFragment.RESULT_UPDATED_IO_NOTE)
                finish()
            }
        }
    }
    return UiPostRequestResponse.Handled(data)
}
