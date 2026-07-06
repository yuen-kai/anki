#!/usr/bin/env python
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Launcher for the Speedrun performance harness.

A sibling of ``launch_anki_for_e2e.py`` that uses a *persistent* ANKI_BASE (so a
seeded 50k collection and sync credentials survive across cold-start restarts)
and a configurable mediasrv port (so it never collides with the developer's
running ``just run`` instance on 40000).

Env:
  ANKI_PERF_BASE   persistent base dir (default: out/speedrun_perf/base)
  ANKI_API_PORT    mediasrv port (default: 40001)
  ANKI_PERF_PROFILE  profile name (default: perf)
"""

from __future__ import annotations

import os
import pickle
import random
import signal
import sqlite3
import subprocess
import sys
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
MEDIASRV_PORT = int(os.environ.get("ANKI_API_PORT", "40001"))
PROFILE = os.environ.get("ANKI_PERF_PROFILE", "perf")
BASE = Path(
    os.environ.get("ANKI_PERF_BASE", str(REPO_ROOT / "out" / "speedrun_perf" / "base"))
)


def _seed_prefs(base: Path) -> None:
    meta = {
        "ver": 0,
        "updates": False,
        "created": int(time.time()),
        "id": random.randrange(0, 2**63),
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
        "syncKey": None,
        "syncMedia": False,
        "autoSync": False,
        "allowHTML": False,
        "importMode": 1,
        "lastColour": "#00f",
        "stripHTML": True,
        "deleteMedia": False,
    }
    db_path = base / "prefs21.db"
    conn = sqlite3.connect(str(db_path))
    conn.execute(
        "create table profiles (name text primary key collate nocase, data blob not null)"
    )
    conn.execute(
        "insert into profiles values ('_global', ?)",
        (pickle.dumps(meta, protocol=4),),
    )
    conn.execute(
        "insert into profiles values (?, ?)",
        (PROFILE, pickle.dumps(profile, protocol=4)),
    )
    conn.commit()
    conn.close()


def main() -> int:
    BASE.mkdir(parents=True, exist_ok=True)
    if not (BASE / "prefs21.db").exists():
        _seed_prefs(BASE)

    env = {
        **os.environ,
        "ANKI_BASE": str(BASE),
        "ANKI_API_PORT": str(MEDIASRV_PORT),
        "ANKI_SINGLE_INSTANCE_KEY": f"anki-perf-{MEDIASRV_PORT}",
        "ANKI_API_HOST": "0.0.0.0",
        "ANKIDEV": "1",
        "PYTHONPYCACHEPREFIX": str(REPO_ROOT / "out" / "pycache"),
        "RUST_BACKTRACE": "1",
        "QT_QPA_PLATFORM": "offscreen",
        "PYTHONUNBUFFERED": "1",
    }
    env.pop("QTWEBENGINE_REMOTE_DEBUGGING", None)
    env.pop("QTWEBENGINE_CHROMIUM_FLAGS", None)
    proc = subprocess.Popen(
        [sys.executable, str(REPO_ROOT / "tools" / "run.py"), "-p", PROFILE],
        env=env,
    )

    def _forward(signum: int, _frame: object) -> None:
        proc.terminate()

    signal.signal(signal.SIGTERM, _forward)
    signal.signal(signal.SIGINT, _forward)
    try:
        return proc.wait()
    except KeyboardInterrupt:
        proc.terminate()
        return proc.wait()


if __name__ == "__main__":
    sys.exit(main())
