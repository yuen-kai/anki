#!/usr/bin/env python
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Speedrun performance RPC harness.

Talks to a running mediasrv (the speedrun_perf_launch instance) over the same
POST /_anki/<method> protobuf surface the Svelte app uses, so it can (a) seed a
large deck and (b) time server-side RPC work as a localhost round-trip.

Usage:
  python speedrun_perf_rpc.py seed --name "Perf 50k" --concepts 50000 [--learn]
  python speedrun_perf_rpc.py list
  python speedrun_perf_rpc.py time <method> <json-body> <N> [warmup]
  python speedrun_perf_rpc.py nextcard <deckId> <N>
  python speedrun_perf_rpc.py answer <deckId> <N>       # times answerCard on due cards

All numbers printed are milliseconds. Localhost HTTP overhead (~0.5-1ms) is
included and noted in the report.
"""

from __future__ import annotations

import json
import os
import statistics
import sys
import time
import urllib.request

sys.path.extend(["pylib", "qt", "out/pylib", "out/qt"])
from anki import (
    generic_pb2,  # noqa: E402
    scheduler_pb2,  # noqa: E402
)

PORT = int(os.environ.get("ANKI_API_PORT", "40001"))
BASE = f"http://127.0.0.1:{PORT}"


def call(method: str, obj: dict | None = None) -> dict:
    body = generic_pb2.Json(json=json.dumps(obj or {}).encode()).SerializeToString()
    req = urllib.request.Request(
        f"{BASE}/_anki/{method}",
        data=body,
        headers={"Content-Type": "application/binary"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        raw = resp.read()
    out = generic_pb2.Json()
    out.ParseFromString(raw)
    return json.loads(out.json.decode() or "{}")


def call_raw(method: str, body: bytes) -> bytes:
    req = urllib.request.Request(
        f"{BASE}/_anki/{method}",
        data=body,
        headers={"Content-Type": "application/binary"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        return resp.read()


def extend_limits(deck_id: str, new_delta: int) -> None:
    msg = scheduler_pb2.ExtendLimitsRequest(
        deck_id=int(deck_id), new_delta=new_delta, review_delta=new_delta
    )
    call_raw("extendLimits", msg.SerializeToString())
    print(f"extended new+review limits by {new_delta} on deck {deck_id}")


def timed(method: str, obj: dict | None = None) -> tuple[float, dict]:
    t0 = time.perf_counter()
    res = call(method, obj)
    return (time.perf_counter() - t0) * 1000.0, res


def pctile(xs: list[float], p: float) -> float:
    if not xs:
        return float("nan")
    s = sorted(xs)
    k = (len(s) - 1) * p
    lo = int(k)
    hi = min(lo + 1, len(s) - 1)
    return s[lo] + (s[hi] - s[lo]) * (k - lo)


def summarize(name: str, samples: list[float]) -> dict:
    res = {
        "metric": name,
        "n": len(samples),
        "p50": round(statistics.median(samples), 2),
        "p95": round(pctile(samples, 0.95), 2),
        "max": round(max(samples), 2),
        "min": round(min(samples), 2),
        "mean": round(statistics.fmean(samples), 2),
    }
    print(json.dumps(res))
    return res


def build_hierarchy(name: str, concepts: int) -> dict:
    # 5 parts x N chapters x 100 concepts. Keep concept bodies small so the blob
    # stays a few MB. Problems empty (not needed to materialize/schedule a card).
    per_leaf = 100
    leaves_total = max(1, concepts // per_leaf)
    parts = 5
    leaves_per_part = max(1, leaves_total // parts)
    children = []
    made = 0
    for p in range(parts):
        chapters = []
        for c in range(leaves_per_part):
            leaf_concepts = []
            for k in range(per_leaf):
                if made >= concepts:
                    break
                cid = f"perf-c{made}"
                leaf_concepts.append(
                    {
                        "id": cid,
                        "title": f"Concept {made}",
                        "content": f"Body for concept {made}.",
                        "problems": [],
                    }
                )
                made += 1
            chapters.append(
                {
                    "id": f"perf-p{p}-ch{c}",
                    "title": f"Chapter {p}.{c}",
                    "children": [],
                    "concepts": leaf_concepts,
                }
            )
        children.append(
            {
                "id": f"perf-part{p}",
                "title": f"Part {p}",
                "children": chapters,
                "concepts": [],
            }
        )
    return {
        "deckId": "new",
        "root": {
            "id": "perf-root",
            "title": name,
            "concepts": [],
            "children": children,
        },
    }


def all_concept_ids(hier: dict) -> list[str]:
    out: list[str] = []

    def walk(node: dict) -> None:
        for c in node.get("concepts", []):
            out.append(c["id"])
        for ch in node.get("children", []):
            walk(ch)

    walk(hier["root"])
    return out


def cmd_seed(name: str, concepts: int, learn: bool) -> None:
    hier = build_hierarchy(name, concepts)
    ids = all_concept_ids(hier)
    print(f"building {len(ids)} concepts under deck {name!r}", flush=True)
    t0 = time.perf_counter()
    saved = call("speedrunSaveHierarchy", hier)
    deck_id = saved.get("deckId", "")
    print(
        f"saveHierarchy -> deckId={deck_id} ({(time.perf_counter() - t0) * 1000:.0f}ms)",
        flush=True,
    )
    t0 = time.perf_counter()
    call("speedrunNextCard", {"deckId": deck_id})
    print(
        f"materialize (nextCard) {(time.perf_counter() - t0) * 1000:.0f}ms", flush=True
    )
    if learn:
        # Flip every concept learning->practicing so nextCard serves review cards
        # (exercises the peek/queue-build path, the steady-state study loop).
        t0 = time.perf_counter()
        call("speedrunRecordLearned", {"deckId": deck_id, "conceptIds": ids})
        print(
            f"recordLearned(all) {(time.perf_counter() - t0) * 1000:.0f}ms", flush=True
        )
    print(json.dumps({"deckId": deck_id, "concepts": len(ids)}))


def cmd_time(method: str, body: dict, n: int, warmup: int) -> None:
    for _ in range(warmup):
        call(method, body)
    samples = [timed(method, body)[0] for _ in range(n)]
    summarize(method, samples)


def main() -> int:
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return 1
    cmd = args[0]
    if cmd == "seed":
        import argparse

        ap = argparse.ArgumentParser()
        ap.add_argument("--name", default="Perf 50k")
        ap.add_argument("--concepts", type=int, default=50000)
        ap.add_argument("--learn", action="store_true")
        ns = ap.parse_args(args[1:])
        cmd_seed(ns.name, ns.concepts, ns.learn)
    elif cmd == "list":
        print(json.dumps(call("speedrunListDecks"), indent=2))
    elif cmd == "time":
        method, body, n = args[1], json.loads(args[2]), int(args[3])
        warmup = int(args[4]) if len(args) > 4 else 3
        cmd_time(method, body, n, warmup)
    elif cmd == "nextcard":
        deck_id, n = args[1], int(args[2])
        cmd_time("speedrunNextCard", {"deckId": deck_id}, n, 3)
    elif cmd == "extend":
        extend_limits(args[1], int(args[2]) if len(args) > 2 else 1000)
    else:
        print(f"unknown command {cmd}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
