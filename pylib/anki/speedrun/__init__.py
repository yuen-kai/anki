# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Speedrun's Python layer.

The study engine (authoring store, concept materialization, scores, mastery
progression) lives in the shared Rust layer (``rslib/src/speedrun``) and is
driven through the backend RPCs, so desktop and AnkiDroid share one engine.

The Python modules here preload authored decks through those same RPCs so the
study screens have real content out of the box: :mod:`anki.speedrun.seed_deck`
(a small biochemistry demo) and :mod:`anki.speedrun.seed_mcat_ch31_35` (a larger
MCAT deck whose data lives in the sibling ``seed_mcat_ch31_35_data`` module).
"""
