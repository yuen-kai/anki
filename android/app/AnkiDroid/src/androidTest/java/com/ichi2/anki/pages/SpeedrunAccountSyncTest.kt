// SPDX-License-Identifier: GPL-3.0-or-later

package com.ichi2.anki.pages

import androidx.fragment.app.FragmentActivity
import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.work.WorkManager
import com.google.protobuf.ByteString
import com.ichi2.anki.CollectionManager.withCol
import com.ichi2.anki.DeckPicker
import com.ichi2.anki.isLoggedIn
import com.ichi2.anki.settings.Prefs
import com.ichi2.anki.tests.InstrumentedTest
import com.ichi2.anki.testutil.GrantStoragePermission.storagePermission
import com.ichi2.anki.testutil.disableIntroductionSlide
import com.ichi2.anki.testutil.discardPreliminaryViews
import com.ichi2.anki.testutil.grantPermissions
import com.ichi2.anki.testutil.notificationPermission
import com.ichi2.anki.updateLogin
import com.ichi2.anki.worker.UniqueWorkNames
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.After
import org.junit.Assume.assumeTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import timber.log.Timber
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import anki.generic.Json as GenericJson

/**
 * On-device verification of the Speedrun account sync flow.
 *
 * Drives the real [handleUiPostRequest] dispatch for `/_anki/speedrunSyncLogin`,
 * `speedrunSyncStatus` and `speedrunSyncNow` against a self-hosted sync server
 * running on the host (reached from the emulator via `10.0.2.2`), and asserts a
 * server-seeded note round-trips onto the device through the native sync engine.
 *
 * Requires a host sync server seeded via pylib:
 *   SYNC_USER1=test:test SYNC_HOST=0.0.0.0 SYNC_PORT=28080 \
 *     SYNC_BASE=/tmp/speedrun-sync-base python -m anki.syncserver
 * and a note whose Front is [SEED_MARKER] uploaded by a pylib client.
 */
@RunWith(AndroidJUnit4::class)
class SpeedrunAccountSyncTest : InstrumentedTest() {
    @get:Rule
    val runtimePermissionRule = grantPermissions(storagePermission, notificationPermission)

    private var scenario: ActivityScenario<DeckPicker>? = null
    private lateinit var activity: FragmentActivity

    @Before
    fun before() {
        // Disable first-run onboarding *before* launching, so DeckPicker is not
        // destroyed by an IntroductionActivity redirect (which breaks onActivity).
        disableIntroductionSlide()
        // start from a signed-out state so the login handler does real work
        updateLogin("", "")
        Prefs.currentSyncUri = ""
    }

    @After
    fun signOut() {
        updateLogin("", "")
        Prefs.currentSyncUri = ""
        scenario?.close()
    }

    /** Launch DeckPicker (a FragmentActivity host) after onboarding is disabled. */
    private fun launchHost() {
        scenario = ActivityScenario.launch(DeckPicker::class.java)
        discardPreliminaryViews()
        scenario!!.onActivity { activity = it }
    }

    /** Wrap a JSON body in the protobuf `generic.Json` envelope, like the frontend does. */
    private fun envelope(build: JSONObject.() -> Unit): ByteArray =
        GenericJson
            .newBuilder()
            .setJson(ByteString.copyFromUtf8(JSONObject().apply(build).toString()))
            .build()
            .toByteArray()

    /** Post through the real URI -> method -> handler dispatch and decode the envelope. */
    private fun post(
        method: String,
        body: ByteArray = ByteArray(0),
    ): JSONObject {
        val methodName = PostRequestUri(AnkiServer.ANKI_PREFIX + method).backendMethodName!!
        val response = runBlocking { activity.handleUiPostRequest(methodName, body) }
        val handled =
            response as? UiPostRequestResponse.Handled
                ?: error("$method should be handled, got $response")
        val json = JSONObject(GenericJson.parseFrom(handled.data).json.toStringUtf8())
        Timber.i("speedrun account: %s -> %s", method, json)
        return json
    }

    @Test
    fun onDeviceAccountSyncFlow() {
        assumeTrue("emulator only (needs host 10.0.2.2)", isEmulator())
        // Skip (don't fail) when no seeded sync server is running, so normal
        // instrumentation runs stay green. See the KDoc for the server setup.
        assumeTrue("sync server not reachable at $ENDPOINT", serverReachable())

        launchHost()

        // 1) login through the fixed handler; hits the real server over the network
        val login =
            post(
                "speedrunSyncLogin",
                envelope {
                    put("username", "test")
                    put("password", "test")
                    put("endpoint", ENDPOINT)
                },
            )
        assertTrue(login.optBoolean("ok"), "login ok: $login")
        assertEquals("test", login.optString("account"))
        assertEquals(ENDPOINT, login.optString("endpoint"))
        assertTrue(isLoggedIn(), "hkey stored after login")

        // 2) status reflects the stored login
        val status = post("speedrunSyncStatus")
        assertTrue(status.optBoolean("loggedIn"), "status loggedIn: $status")
        assertEquals("test", status.optString("account"))
        assertEquals(ENDPOINT, status.optString("endpoint"))

        // 3) data round-trip through the real account handler: `speedrunSyncNow`
        //    must perform the first-time full sync itself, so a server-seeded note
        //    lands on the device with no manual download. This is the regression:
        //    the handler used to fire the background SyncWorker, which skips
        //    one-way syncs, so a fresh phone never picked up the server's data.
        val now = post("speedrunSyncNow")
        Timber.i("speedrun account: syncNow -> %s", now)
        if (now.optBoolean("conflict")) {
            // The device already had its own collection: adopt the server's copy.
            val resolved = post("speedrunSyncNow", envelope { put("resolve", "download") })
            assertTrue(resolved.optBoolean("ok"), "resolve download ok: $resolved")
        } else {
            assertTrue(now.optBoolean("ok"), "syncNow ok: $now")
        }
        val seeded = runBlocking { withCol { findNotes(SEED_MARKER) } }
        Timber.i("speedrun account: notes matching %s = %s", SEED_MARKER, seeded)
        assertTrue(seeded.isNotEmpty(), "seeded note present on device after speedrunSyncNow: $seeded")

        // 4) let the background media sync finish so it does not race teardown.
        awaitUniqueWorkFinished(UniqueWorkNames.SYNC_MEDIA)
    }

    /** True when the host sync server answers on 10.0.2.2:28080 (emulator -> host loopback). */
    private fun serverReachable(): Boolean =
        try {
            java.net.Socket().use {
                it.connect(java.net.InetSocketAddress("10.0.2.2", 28080), 2000)
                true
            }
        } catch (_: Exception) {
            false
        }

    /** Poll WorkManager until the given unique work leaves the running set (best effort). */
    private fun awaitUniqueWorkFinished(uniqueName: String) {
        val wm = WorkManager.getInstance(activity.applicationContext)
        val deadline = System.currentTimeMillis() + 30_000
        while (System.currentTimeMillis() < deadline) {
            val infos = wm.getWorkInfosForUniqueWork(uniqueName).get()
            val states = infos.map { it.state }
            if (infos.isEmpty() || states.all { it.isFinished }) {
                Timber.i("speedrun account: %s states=%s", uniqueName, states)
                return
            }
            Thread.sleep(250)
        }
        Timber.w("speedrun account: %s did not finish in time; cancelling", uniqueName)
        wm.cancelUniqueWork(uniqueName)
    }

    companion object {
        private const val ENDPOINT = "http://10.0.2.2:28080/"
        private const val SEED_MARKER = "SERVER_SEED_NOTE"
    }
}
