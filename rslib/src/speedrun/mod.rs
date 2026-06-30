// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Speedrun: MCAT study extensions on top of Anki.
//!
//! Phase 1A ships only the foundation the rest of the fork stands on: the AAMC
//! MCAT topic taxonomy (a seed subset) plus pure coverage/weakness helpers.
//! There is deliberately no protobuf and no scheduler/queue code here; later
//! phases build those against the contracts defined in [`taxonomy`].

pub mod taxonomy;
