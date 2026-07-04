# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Speedrun's Python layer.

The study engine (authoring store, concept materialization, scores, mastery
progression) lives in the shared Rust layer (``rslib/src/speedrun``) and is
driven through the backend RPCs, so desktop and AnkiDroid share one engine.

The preloaded demo + MCAT decks are seeded from that shared layer too
(``rslib/src/speedrun/seed.rs``, exposed as ``SpeedrunEnsureSeeded``), so both
hosts get content out of the box with no Python seed code.
"""
