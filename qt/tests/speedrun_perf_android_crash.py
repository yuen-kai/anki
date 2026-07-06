#!/usr/bin/env python
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Android crash / corruption test for the Speedrun collection (target 9).

Self-contained + safe: disables device network, pushes a *test* 50k collection
(never the user's), then for N trials drives study writes in the WebView and
hard-kills the app (am force-stop) mid-write; reopens and verifies the pulled
collection with SQLite integrity_check (registering Anki's `unicase` collation)
and that the app relaunches. Finally RESTORES the user's original collection and
re-enables network.

Requires: adb on PATH; out/speedrun_perf/perf50k_consolidated.anki2 (test 50k)
and out/speedrun_perf/android_orig_collection.anki2 (user's backup) present.

Env: SR_ACRASH_N (default 10).
"""

from __future__ import annotations

import json
import os
import random
import sqlite3
import subprocess
import sys
import time

PKG = "com.ichi2.anki.debug"
ACT = f"{PKG}/com.ichi2.anki.IntentHandler"
DIR = "/storage/emulated/0/AnkiDroid"
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PERF = os.path.join(REPO, "out", "speedrun_perf")
TEST_COL = os.path.join(PERF, "perf50k_consolidated.anki2")
ORIG_COL = os.path.join(PERF, "android_orig_collection.anki2")
TMP = os.path.join(PERF, "android_pull")


def adb(*args: str, timeout: float = 60) -> str:
    return subprocess.run(
        ["adb", *args], capture_output=True, text=True, timeout=timeout
    ).stdout


def shell(cmd: str, timeout: float = 60) -> str:
    return adb("shell", cmd, timeout=timeout)


def launch() -> None:
    shell(
        f"am start -n {ACT} -a android.intent.action.MAIN -c android.intent.category.LAUNCHER"
    )


def force_stop() -> None:
    shell(f"am force-stop {PKG}")


def pid() -> str:
    return shell(f"pidof {PKG}").strip()


def _unicase(a: str, b: str) -> int:
    x, y = a.casefold(), b.casefold()
    return (x > y) - (x < y)


def integrity_ok() -> tuple[bool, str]:
    os.makedirs(TMP, exist_ok=True)
    for suffix in ("", "-wal", "-shm"):
        try:
            os.remove(os.path.join(TMP, "collection.anki2" + suffix))
        except OSError:
            pass
    adb("pull", f"{DIR}/collection.anki2", os.path.join(TMP, "collection.anki2"))
    adb(
        "pull", f"{DIR}/collection.anki2-wal", os.path.join(TMP, "collection.anki2-wal")
    )
    con = None
    try:
        con = sqlite3.connect(os.path.join(TMP, "collection.anki2"), timeout=15)
        con.create_collation("unicase", _unicase)
        detail = ";".join(
            r[0] for r in con.execute("pragma integrity_check").fetchall()
        )
        cards = con.execute("select count(*) from cards").fetchone()[0]
        return (detail == "ok", f"integrity={detail[:80]} cards={cards}")
    except Exception as e:
        return (False, f"sqlite-error:{e}")
    finally:
        if con is not None:
            con.close()


def tap(x: int, y: int) -> None:
    shell(f"input tap {x} {y}")


def drive_writes() -> None:
    """Best-effort: enter a study session and spam the primary action column so
    learning/grading write paths (recordLearned / answerCard) fire. Device is
    1080x2400. The first deck's Study button sits ~ (265, 874)."""
    tap(265, 874)  # Study on first deck
    time.sleep(2.5)
    tap(540, 2150)  # "Start studying" (bottom primary)
    time.sleep(2.0)
    for _ in range(8):  # drive through reveal/grade controls
        tap(540, 1500)
        tap(540, 1800)
        tap(270, 1500)
        time.sleep(0.25)


def main() -> int:
    n = int(os.environ.get("SR_ACRASH_N", "10"))
    if not (os.path.exists(TEST_COL) and os.path.exists(ORIG_COL)):
        print("missing test/orig collection artifacts", file=sys.stderr)
        return 1
    # Safety: network off + push the TEST collection.
    shell("svc wifi disable")
    shell("svc data disable")
    force_stop()
    time.sleep(1)
    shell(f"rm -f {DIR}/collection.anki2-wal {DIR}/collection.anki2-shm")
    adb("push", TEST_COL, f"{DIR}/collection.anki2")

    corrupt = 0
    reopen_fail = 0
    results = []
    try:
        for i in range(n):
            launch()
            time.sleep(4.5)  # decks WebView loads
            t = threading_start(drive_writes)
            time.sleep(random.uniform(0.4, 2.2))  # kill mid-write
            force_stop()
            t.join(timeout=3)
            ok, detail = integrity_ok()
            # Reopen and confirm the app process comes back up and stays.
            launch()
            time.sleep(6)
            reopened = bool(pid())
            if not ok:
                corrupt += 1
            if not reopened:
                reopen_fail += 1
            force_stop()
            results.append(
                {"trial": i, "integrity_ok": ok, "reopened": reopened, "detail": detail}
            )
            print(
                f"trial {i}: integrity_ok={ok} reopened={reopened} ({detail})",
                flush=True,
            )
    finally:
        # RESTORE the user's collection + network no matter what.
        force_stop()
        time.sleep(1)
        shell(f"rm -f {DIR}/collection.anki2-wal {DIR}/collection.anki2-shm")
        adb("push", ORIG_COL, f"{DIR}/collection.anki2")
        shell("svc wifi enable")
        shell("svc data enable")
        print("restored user's collection + network", flush=True)

    print(
        json.dumps(
            {
                "metric": "android_crash_corruption",
                "trials": n,
                "corrupted": corrupt,
                "reopen_failures": reopen_fail,
            }
        )
    )
    return 0


def threading_start(fn):
    import threading

    t = threading.Thread(target=fn)
    t.start()
    return t


if __name__ == "__main__":
    sys.exit(main())
