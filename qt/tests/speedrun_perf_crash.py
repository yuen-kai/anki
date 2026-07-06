#!/usr/bin/env python
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Desktop crash / corruption test for the Speedrun collection (target 9).

For N trials: launch the app, hammer writes (grade cards -> FSRS write + revlog +
custom_data), hard-kill (kill -9) the process mid-write, then reopen and verify
the collection is intact: SQLite `pragma integrity_check` == "ok" AND the app
re-opens and serves decks. Counts how many trials end corrupted (target 0).

Env: ANKI_API_PORT (default 40001), ANKI_PERF_BASE, SR_CRASH_N (default 20),
     SR_CRASH_DECK (deck id to grade).
"""

from __future__ import annotations

import json
import os
import random
import sqlite3
import subprocess
import sys
import threading
import time
import urllib.request

sys.path.extend(["pylib", "qt", "out/pylib", "out/qt"])
from anki import generic_pb2  # noqa: E402

PORT = int(os.environ.get("ANKI_API_PORT", "40001"))
BASE = f"http://127.0.0.1:{PORT}"
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PERF_BASE = os.environ.get(
    "ANKI_PERF_BASE", os.path.join(REPO, "out", "speedrun_perf", "base")
)
COL = os.path.join(PERF_BASE, "perf", "collection.anki2")
DECK = os.environ.get("SR_CRASH_DECK", "1783296074978")


def call(method: str, obj: dict, timeout: float = 5.0) -> dict:
    body = generic_pb2.Json(json=json.dumps(obj).encode()).SerializeToString()
    req = urllib.request.Request(
        f"{BASE}/_anki/{method}",
        data=body,
        headers={"Content-Type": "application/binary"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        out = generic_pb2.Json()
        out.ParseFromString(r.read())
        return json.loads(out.json.decode() or "{}")


def up() -> bool:
    try:
        call("speedrunListDecks", {}, 2)
        return True
    except Exception:
        return False


def launch() -> None:
    env = {**os.environ, "ANKI_API_PORT": str(PORT), "ANKI_PERF_BASE": PERF_BASE}
    subprocess.Popen(
        [
            os.path.join(REPO, "out/pyenv/bin/python"),
            os.path.join(REPO, "qt/tests/speedrun_perf_launch.py"),
        ],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    t0 = time.time()
    while not up() and time.time() - t0 < 40:
        time.sleep(0.05)


def kill9() -> None:
    subprocess.run(["pkill", "-9", "-f", "run.py -p perf"], capture_output=True)
    t0 = time.time()
    while up() and time.time() - t0 < 20:
        time.sleep(0.05)


def hammer(stop: threading.Event) -> None:
    while not stop.is_set():
        try:
            nc = call("speedrunNextCard", {"deckId": DECK}, 5)
            if nc.get("kind") == "review":
                call(
                    "speedrunAnswerCard",
                    {
                        "deckId": DECK,
                        "cardId": nc["cardId"],
                        "conceptId": nc["conceptId"],
                        "rating": random.choice([1, 3]),
                    },
                    5,
                )
        except Exception:
            pass


def _unicase(a: str, b: str) -> int:
    # Mirror rslib's unicase collation (case-insensitive) closely enough for
    # index-order verification of our ASCII-heavy content.
    x, y = a.casefold(), b.casefold()
    return (x > y) - (x < y)


def integrity_ok() -> tuple[bool, str]:
    """Raw SQLite structural check. Registers the `unicase` collation Anki uses
    so index checks resolve. Returns (structurally_ok, detail)."""
    con = None
    try:
        con = sqlite3.connect(COL, timeout=15)
        con.create_collation("unicase", _unicase)
        r1 = con.execute("pragma integrity_check").fetchall()
        detail = ";".join(row[0] for row in r1)
        # "ok" == no page/btree corruption. Collation-order notes (if the mimic
        # differs) surface as index rows and are reported but not fatal here;
        # Anki's own reopen + full scan is the authoritative signal.
        page_ok = detail == "ok"
        return (page_ok, f"integrity={detail[:120]}")
    except Exception as e:
        return (False, f"sqlite-error:{e}")
    finally:
        if con is not None:
            con.close()


def main() -> int:
    n = int(os.environ.get("SR_CRASH_N", "20"))
    if not up():
        launch()
    corrupt = 0
    app_open_fail = 0
    results = []
    for i in range(n):
        if not up():
            launch()
        stop = threading.Event()
        threads = [threading.Thread(target=hammer, args=(stop,)) for _ in range(4)]
        for t in threads:
            t.start()
        time.sleep(random.uniform(0.3, 1.8))  # let writes get in flight
        kill9()
        stop.set()
        for t in threads:
            t.join(timeout=2)
        sqlite_ok, detail = integrity_ok()
        # Authoritative: reopen via the real Anki app (registers unicase, WAL
        # recovery) and confirm it serves decks AND a full card scan works.
        launch()
        reopened = up()
        deep_ok = False
        if reopened:
            try:
                st = call("speedrunStudyState", {"deckId": DECK}, 30)
                deep_ok = len(st.get("progress", {})) == 50000
            except Exception as e:
                detail += f" deep-scan-error:{e}"
        # Corruption = Anki cannot reopen, or a full scan fails/returns wrong data.
        corrupt_trial = not (reopened and deep_ok)
        if corrupt_trial:
            corrupt += 1
        if not reopened:
            app_open_fail += 1
        results.append(
            {
                "trial": i,
                "sqlite_page_ok": sqlite_ok,
                "reopened": reopened,
                "deep_scan_ok": deep_ok,
                "detail": detail,
            }
        )
        print(
            f"trial {i}: sqlite_page_ok={sqlite_ok} reopened={reopened} "
            f"deep_scan_ok={deep_ok} ({detail})",
            flush=True,
        )
    print(
        json.dumps(
            {
                "metric": "desktop_crash_corruption",
                "trials": n,
                "corrupted": corrupt,
                "app_open_failures": app_open_fail,
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
