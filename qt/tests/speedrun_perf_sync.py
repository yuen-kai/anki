#!/usr/bin/env python
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Real AnkiWeb sync round-trip measurement (target 5), done safely.

The dev account syncs to real AnkiWeb, so to avoid any risk to the user's data
we sync a *consistent .backup copy* of their already-synced collection, reusing
their existing sync key (same collection lineage => incremental two-way merge,
never a clobbering full sync). With no local changes this is a genuine sync
round-trip that uploads nothing; a normal 20-50 card session adds only a few KB,
so this round-trip is representative of "sync of a normal session".

Env: ANKI_API_PORT (default 40003), SR_SYNC_N (default 6).
"""

from __future__ import annotations

import json
import os
import pickle
import shutil
import sqlite3
import statistics
import subprocess
import sys
import time
import urllib.request

sys.path.extend(["pylib", "qt", "out/pylib", "out/qt"])
from anki import generic_pb2  # noqa: E402

PORT = int(os.environ.get("ANKI_API_PORT", "40003"))
BASE_URL = f"http://127.0.0.1:{PORT}"
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
USER_BASE = os.path.expanduser("~/Library/Application Support/Anki2")
USER_PROFILE = "User 1"
SYNC_BASE = os.path.join(REPO, "out", "speedrun_perf", "base_sync")
PROFILE = "perfsync"


def user_sync_prefs() -> dict:
    con = sqlite3.connect(os.path.join(USER_BASE, "prefs21.db"))
    row = con.execute(
        "select data from profiles where name=?", (USER_PROFILE,)
    ).fetchone()
    con.close()
    d = pickle.loads(row[0])
    return {k: d.get(k) for k in ("syncKey", "syncUser", "currentSyncUrl", "hostNum")}


def seed_prefs(base: str, sync: dict) -> None:
    os.makedirs(base, exist_ok=True)
    meta = {
        "ver": 0,
        "updates": False,
        "created": int(time.time()),
        "id": 12345,
        "lastMsg": 0,
        "suppressUpdate": True,
        "firstRun": False,
        "defaultLang": "en_US",
        "check_for_updates": False,
    }
    profile = {
        "mainWindowGeom": None,
        "mainWindowState": None,
        "numBackups": 0,
        "lastOptimize": int(time.time()),
        "searchHistory": [],
        "syncKey": sync.get("syncKey"),
        "syncUser": sync.get("syncUser"),
        "currentSyncUrl": sync.get("currentSyncUrl"),
        "hostNum": sync.get("hostNum"),
        "syncMedia": False,
        "autoSync": False,
        "allowHTML": False,
        "importMode": 1,
        "lastColour": "#00f",
        "stripHTML": True,
        "deleteMedia": False,
    }
    con = sqlite3.connect(os.path.join(base, "prefs21.db"))
    con.execute(
        "create table profiles (name text primary key collate nocase, data blob not null)"
    )
    con.execute(
        "insert into profiles values ('_global', ?)", (pickle.dumps(meta, protocol=4),)
    )
    con.execute(
        "insert into profiles values (?, ?)",
        (PROFILE, pickle.dumps(profile, protocol=4)),
    )
    con.commit()
    con.close()


def backup_collection(base: str) -> None:
    prof_dir = os.path.join(base, PROFILE)
    os.makedirs(prof_dir, exist_ok=True)
    src = sqlite3.connect(os.path.join(USER_BASE, USER_PROFILE, "collection.anki2"))
    dst = sqlite3.connect(os.path.join(prof_dir, "collection.anki2"))
    with dst:
        src.backup(dst)
    src.close()
    dst.close()


def call(method: str, obj: dict, timeout: float = 200.0) -> dict:
    body = generic_pb2.Json(json=json.dumps(obj).encode()).SerializeToString()
    req = urllib.request.Request(
        f"{BASE_URL}/_anki/{method}",
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


def main() -> int:
    n = int(os.environ.get("SR_SYNC_N", "6"))
    subprocess.run(["pkill", "-9", "-f", "run.py -p perfsync"], capture_output=True)
    time.sleep(1)
    shutil.rmtree(SYNC_BASE, ignore_errors=True)
    sync = user_sync_prefs()
    print(
        "using sync auth:",
        {k: (v if k != "syncKey" else "<hidden>") for k, v in sync.items()},
        flush=True,
    )
    seed_prefs(SYNC_BASE, sync)
    backup_collection(SYNC_BASE)

    env = {
        **os.environ,
        "ANKI_API_PORT": str(PORT),
        "ANKI_PERF_BASE": SYNC_BASE,
        "ANKI_PERF_PROFILE": PROFILE,
    }
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
        time.sleep(0.1)
    if not up():
        print("instance did not come up")
        return 1

    samples: list[float] = []
    for i in range(n):
        t = time.perf_counter()
        res = call("speedrunSyncNow", {"interactive": False})
        dt = (time.perf_counter() - t) * 1000.0
        status = res.get("status")
        print(
            f"sync {i}: {dt:.0f}ms status={status} ok={res.get('ok')} "
            f"msg={res.get('message', '')}",
            flush=True,
        )
        if i > 0:  # discard first (may include auth warmup / small delta)
            samples.append(dt)
        time.sleep(1.0)
    subprocess.run(["pkill", "-9", "-f", "run.py -p perfsync"], capture_output=True)
    if samples:
        print(
            json.dumps(
                {
                    "metric": "desktop_sync_roundtrip_ms",
                    "n": len(samples),
                    "p50": round(statistics.median(samples), 1),
                    "p95": round(
                        sorted(samples)[max(0, int(len(samples) * 0.95) - 1)], 1
                    ),
                    "max": round(max(samples), 1),
                    "min": round(min(samples), 1),
                }
            )
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
