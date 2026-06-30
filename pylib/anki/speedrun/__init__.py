# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Speedrun's application-teaching content layer.

This package holds the no-AI Learn content from spec-study-model: the
contrasting-cases concept cards and the principle-first scaffold, shipped as
Anki note types so they render on desktop and AnkiDroid alike (decision D19).

The ``feedback`` and ``seed_content`` modules are deliberately pure Python with
no ``anki`` import, so the content and the static feedback map can be unit
tested without building the Rust backend. ``notetypes`` is the only module that
touches a collection; keep heavyweight imports out of this ``__init__`` so that
``import anki.speedrun.feedback`` stays backend-free.
"""
