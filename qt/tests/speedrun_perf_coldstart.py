#!/usr/bin/env python
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Desktop cold-start harness for the Speedrun decks screen.

For N trials: force-stop any perf instance, launch a fresh one, and time from
process spawn until the collection is loaded and the decks RPC serves
(a proxy for "first interactive /speedrun-decks", excluding the browser paint
which adds the ~100-200ms measured separately by the dashboard e2e test).
Also samples steady-state RSS after load.

Env: ANKI_API_PORT (default 40001), ANKI_PERF_BASE, SR_COLD_N (default 6).
"""

from __future__ import annotations

import json
import os
import signal
import statistics
import subprocess
import sys
import time
import urllib.request

sys.path.extend(["pylib", "qt", "out/pylib", "out/qt"])
from anki import generic_pb2  # noqa: E402

PORT = int(os.environ.get("ANKI_API_PORT", "40001"))
BASE = f"http://127.0.0.1:{PORT}"
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def list_decks_ok() -> bool:
    try:
        body = generic_pb2.Json(json=b"{}").SerializeToString()
        req = urllib.request.Request(
            f"{BASE}/_anki/speedrunListDecks",
            data=body,
            headers={"Content-Type": "application/binary"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=2) as r:
            out = generic_pb2.Json()
            out.ParseFromString(r.read())
            json.loads(out.json.decode() or "[]")
            return True
    except Exception:
        return False


def pctile(xs, p):
    s = sorted(xs)
    k = (len(s) - 1) * p
    lo = int(k)
    hi = min(lo + 1, len(s) - 1)
    return s[lo] + (s[hi] - s[lo]) * (k - lo)


def force_stop() -> None:
    subprocess.run(["pkill", "-9", "-f", "run.py -p perf"], capture_output=True)
    for _ in range(50):
        if not list_decks_ok():
            return
        time.sleep(0.1)


def anki_pid() -> int:
    out = subprocess.run(
        ["pgrep", "-f", "run.py -p perf"], capture_output=True, text=True
    )
    pids = [int(x) for x in out.stdout.split()]
    return pids[0] if pids else -1


def rss_mb(_pid: int) -> int:
    pid = anki_pid()
    if pid < 0:
        return -1
    out = subprocess.run(
        ["ps", "-o", "rss=", "-p", str(pid)], capture_output=True, text=True
    )
    try:
        return int(out.stdout.strip()) // 1024
    except ValueError:
        return -1


def main() -> int:
    n = int(os.environ.get("SR_COLD_N", "6"))
    env = {**os.environ, "ANKI_API_PORT": str(PORT)}
    py = os.path.join(REPO, "out", "pyenv", "bin", "python")
    launcher = os.path.join(REPO, "qt", "tests", "speedrun_perf_launch.py")
    samples: list[float] = []
    rsses: list[int] = []
    for i in range(n):
        force_stop()
        time.sleep(1.0)
        t0 = time.perf_counter()
        proc = subprocess.Popen(
            [py, launcher],
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        while not list_decks_ok():
            if time.perf_counter() - t0 > 30:
                print("timeout waiting for decks", flush=True)
                break
            time.sleep(0.01)
        dt = (time.perf_counter() - t0) * 1000.0
        samples.append(dt)
        time.sleep(1.5)  # let it settle
        rsses.append(rss_mb(proc.pid))
        print(f"trial {i}: cold_ready={dt:.0f}ms rss={rsses[-1]}MB", flush=True)
    print(
        json.dumps(
            {
                "metric": "desktop_cold_start_ms",
                "n": len(samples),
                "p50": round(statistics.median(samples), 1),
                "p95": round(pctile(samples, 0.95), 1),
                "max": round(max(samples), 1),
                "min": round(min(samples), 1),
                "rss_mb_median": int(statistics.median(rsses)) if rsses else -1,
                "rss_mb_max": max(rsses) if rsses else -1,
            }
        )
    )
    # leave a fresh instance running for subsequent measurements
    force_stop()
    time.sleep(1.0)
    subprocess.Popen(
        [py, launcher],
        env=env,
        stdout=open(os.path.join(REPO, "out", "speedrun_perf_launch.log"), "w"),
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    return 0


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
    sys.exit(main())
