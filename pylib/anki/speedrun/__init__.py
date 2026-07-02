# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Speedrun's Python layer.

The study engine (authoring store, concept materialization, scores, mastery
progression) lives in the shared Rust layer (``rslib/src/speedrun``) and is
driven through the backend RPCs, so desktop and AnkiDroid share one engine.

The only Python module here is :mod:`anki.speedrun.seed_deck`, which preloads a
complete authored demo deck through those same RPCs so the study screens have
real content out of the box.
"""
