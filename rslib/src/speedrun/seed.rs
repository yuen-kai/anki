// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Speedrun preloaded decks, seeded through the shared authoring engine.
//!
//! Both hosts (the Qt desktop app and the AnkiDroid shell) call
//! [`Collection::speedrun_ensure_seeded`] so the study/review screens have real
//! content out of the box: a small biochemistry demo and a larger MCAT
//! chapters 31-35 deck. The deck data lives as JSON bundled at compile time
//! (`seed_data/*.json`, one authoring-store blob each), so there is no
//! per-platform seed code and no second authoring path: seeding reuses the same
//! `speedrun_save_hierarchy` (create the backing deck + store the blob) and
//! `speedrun_next_card` (materialize one FSRS card per concept) that the UI
//! drives.
//!
//! Seeding is idempotent and fail-open per deck, keyed on a per-deck **content
//! signature** — a stable hash of the bundled blob (normalized to ignore its
//! `deckId`) recorded under [`SEED_SIGNATURE_CONFIG_KEY`] whenever a deck is
//! seeded or refreshed:
//!
//! - A deck that does not yet exist (matched by name) is created, reconciled,
//!   and its signature recorded.
//! - A deck that already exists is refreshed to the current bundle only when the
//!   bundle's content **changed** since it was last seeded (the recorded
//!   signature differs from the current bundle's). Re-storing the bundle under
//!   the same deck id keeps every matching concept's card — reconcile is keyed
//!   by `conceptId`, so study progress survives — and materializes any new
//!   concepts, so rewritten titles/content/problems/image refs appear. When the
//!   signature already matches, the deck is left untouched, so an unchanged
//!   bundle never clobbers a deck the user edited in-app.
//! - A deck seeded before this mechanism has no recorded signature, so it
//!   differs from the current bundle and refreshes once — migrating stale
//!   preloaded content (e.g. an older MCAT deck), of which the old
//!   "fewer concepts" backfill is now just a subcase — then becomes idempotent.
//!
//! So a reseed on every collection open propagates a changed bundle to its deck
//! (preserving FSRS progress on surviving concepts) while leaving an unchanged
//! bundle — and the decks the user has edited — alone.

use std::collections::HashMap;

use serde_json::json;
use serde_json::Value;
use sha1::Digest;
use sha1::Sha1;

use crate::prelude::*;

/// Collection-config key holding `{ deckId -> content signature }` for the
/// bundle last seeded into each deck. Lets [`Collection::seed_one_deck`] refresh
/// a deck only when the bundle's content actually changed (see the module docs),
/// so an unchanged bundle never reseeds and never clobbers in-app edits.
const SEED_SIGNATURE_CONFIG_KEY: &str = "speedrun_seed_signature";

/// The bundled decks as authoring-store blobs (`{ deckId, root }`). The deck
/// name is the blob's root title, so the JSON is the single source of truth.
const BUNDLED_DECKS: [&str; 2] = [
    include_str!("seed_data/demo_biochem.json"),
    include_str!("seed_data/mcat_ch31_35.json"),
];

// `SEED_MEDIA: &[(&str, &[u8])]` — the bundled Speedrun images (name + bytes),
// generated at build time from `seed_data/mcat_ch31_35_media/*.png` (see
// `rslib/build.rs`). Empty when the folder holds no PNGs. `include_str!` can't
// embed binaries, so the build script emits `include_bytes!` entries.
include!(concat!(env!("OUT_DIR"), "/speedrun_seed_media.rs"));

/// What [`Collection::seed_one_deck`] did with one bundled deck.
enum SeedOutcome {
    /// The deck did not exist and was created.
    Created(String),
    /// The deck existed but was out of date and was refreshed to the bundle.
    Updated(String),
    /// The deck already existed and was current (or the blob had no name/id).
    Unchanged,
}

/// A stable content signature for a bundled blob, independent of the deck id it
/// is stored under: the blob is normalized (its `deckId` dropped) and hashed, so
/// two blobs that differ only by `deckId` share a signature. Recorded per deck
/// at seed time so a later open can tell "the bundle's content changed since we
/// seeded this deck" (signature differs — refresh) from "the user edited the
/// deck in-app" (signature still matches — leave alone). serde_json serializes
/// object keys sorted (no `preserve_order`), so the normalized form is stable.
fn bundle_signature(hierarchy: &Value) -> String {
    let mut normalized = hierarchy.clone();
    if let Some(obj) = normalized.as_object_mut() {
        obj.remove("deckId");
    }
    let serialized = serde_json::to_vec(&normalized).unwrap_or_default();
    hex::encode(Sha1::digest(&serialized))
}

/// Total concepts across a hierarchy blob's tree (`{ root: { concepts, children
/// } }`), summed over every node.
#[cfg(test)]
fn count_concepts(hierarchy: &Value) -> usize {
    fn walk(node: &Value) -> usize {
        let here = node
            .get("concepts")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0);
        let below: usize = node
            .get("children")
            .and_then(Value::as_array)
            .map(|kids| kids.iter().map(walk).sum())
            .unwrap_or(0);
        here + below
    }
    hierarchy.get("root").map(walk).unwrap_or(0)
}

impl Collection {
    /// Create or backfill every bundled Speedrun deck. Idempotent, so it is
    /// safe to call on every collection open. Returns `{ seeded: [name],
    /// updated: [name] }` — decks freshly created and decks refreshed from
    /// an older bundle, respectively (both empty when everything was
    /// already current).
    pub(crate) fn speedrun_ensure_seeded(&mut self) -> Result<Value> {
        // Copy bundled images first so a deck's figures resolve as soon as its
        // cards materialize. Runs unconditionally (not per deck) so images added
        // to a later bundle appear for already-seeded decks after a reload too.
        self.copy_seed_media()?;

        let mut seeded: Vec<String> = Vec::new();
        let mut updated: Vec<String> = Vec::new();
        for blob in BUNDLED_DECKS {
            match self.seed_one_deck(blob)? {
                SeedOutcome::Created(name) => seeded.push(name),
                SeedOutcome::Updated(name) => updated.push(name),
                SeedOutcome::Unchanged => {}
            }
        }
        Ok(json!({ "seeded": seeded, "updated": updated }))
    }

    /// Copy every bundled Speedrun image ([`SEED_MEDIA`]) into the collection's
    /// media folder by exact filename, idempotently (see
    /// [`crate::media::MediaManager::add_or_replace_file`]). A no-op when nothing
    /// is bundled or the collection has no media folder (an in-memory test
    /// collection), so seeding still works without media configured.
    fn copy_seed_media(&self) -> Result<()> {
        if SEED_MEDIA.is_empty() || self.media_folder.as_os_str().is_empty() {
            return Ok(());
        }
        let media = self.media()?;
        for (name, data) in SEED_MEDIA {
            media.add_or_replace_file(name, data)?;
        }
        Ok(())
    }

    /// Create a bundled deck, or refresh it when the bundle's content changed
    /// since it was last seeded (see the module docs). Reconciles either way so
    /// the concepts are materialized into studyable cards, and records the
    /// bundle's content signature so the next open can skip an unchanged bundle.
    fn seed_one_deck(&mut self, blob: &str) -> Result<SeedOutcome> {
        let mut hierarchy: Value = serde_json::from_str(blob)?;
        let name = hierarchy
            .get("root")
            .and_then(|root| root.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            return Ok(SeedOutcome::Unchanged);
        }
        let signature = bundle_signature(&hierarchy);

        let Some(did) = self.get_deck_id(&name)? else {
            // Fresh seed: create the backing deck + store the blob, reconcile,
            // then record the bundle's signature.
            let saved = self.speedrun_save_hierarchy(hierarchy)?;
            let deck_id = saved
                .get("deckId")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if deck_id.is_empty() {
                return Ok(SeedOutcome::Unchanged);
            }
            let did = DeckId(crate::speedrun::parse_id(&deck_id, "deckId")?);
            self.speedrun_next_card(did)?;
            self.record_seed_signature(did, &signature)?;
            return Ok(SeedOutcome::Created(name));
        };

        // The deck exists. Leave it alone when the bundle's content is unchanged
        // since we last seeded it (recorded signature matches), so an unchanged
        // bundle never clobbers in-app edits. Otherwise — the bundle changed, or
        // the deck predates this mechanism (no recorded signature) — refresh it
        // to the bundle in place and record the new signature. Reconcile is
        // keyed by conceptId, so surviving concepts keep their cards/progress.
        if self.seed_signature_map().get(&did.0.to_string()) == Some(&signature) {
            return Ok(SeedOutcome::Unchanged);
        }
        hierarchy["deckId"] = json!(did.0.to_string());
        self.speedrun_save_hierarchy(hierarchy)?;
        self.speedrun_next_card(did)?;
        self.record_seed_signature(did, &signature)?;
        Ok(SeedOutcome::Updated(name))
    }

    /// The recorded `{ deckId -> bundle content signature }` map (empty when no
    /// deck has been seeded yet).
    fn seed_signature_map(&self) -> HashMap<String, String> {
        self.get_config_optional(SEED_SIGNATURE_CONFIG_KEY)
            .unwrap_or_default()
    }

    /// Record the content signature of the bundle just seeded into `deck_id`.
    /// Non-undoable, matching the authoring store's bespoke config writes.
    fn record_seed_signature(&mut self, deck_id: DeckId, signature: &str) -> Result<()> {
        let mut map = self.seed_signature_map();
        map.insert(deck_id.0.to_string(), signature.to_string());
        self.set_config_json(SEED_SIGNATURE_CONFIG_KEY, &map, false)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchNode;
    use crate::search::SortMode;
    use crate::tests::open_fs_test_collection;

    fn deck_names() -> Vec<String> {
        BUNDLED_DECKS
            .iter()
            .map(|blob| {
                serde_json::from_str::<Value>(blob).unwrap()["root"]["title"]
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }

    /// The MCAT deck's bundled name (the second bundled deck).
    fn mcat_name() -> String {
        deck_names()[1].clone()
    }

    fn card_count(col: &mut Collection, did: DeckId) -> usize {
        col.search_cards(SearchNode::from_deck_id(did, true), SortMode::NoOrder)
            .unwrap()
            .len()
    }

    fn json_names(value: &Value, key: &str) -> Vec<String> {
        value[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    fn count_problems(hierarchy: &Value) -> usize {
        fn walk(node: &Value) -> usize {
            let here: usize = node
                .get("concepts")
                .and_then(Value::as_array)
                .map(|cs| {
                    cs.iter()
                        .map(|c| {
                            c.get("problems")
                                .and_then(Value::as_array)
                                .map(Vec::len)
                                .unwrap_or(0)
                        })
                        .sum()
                })
                .unwrap_or(0);
            let below: usize = node
                .get("children")
                .and_then(Value::as_array)
                .map(|kids| kids.iter().map(walk).sum())
                .unwrap_or(0);
            here + below
        }
        hierarchy.get("root").map(walk).unwrap_or(0)
    }

    /// `(leaf title, concept count)` for every leaf topic (a node with no
    /// children), in tree order.
    fn leaf_concept_counts(hierarchy: &Value) -> Vec<(String, usize)> {
        fn walk(node: &Value, out: &mut Vec<(String, usize)>) {
            match node.get("children").and_then(Value::as_array) {
                Some(kids) if !kids.is_empty() => kids.iter().for_each(|k| walk(k, out)),
                _ => out.push((
                    node.get("title")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    node.get("concepts")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        .unwrap_or(0),
                )),
            }
        }
        let mut out = Vec::new();
        if let Some(root) = hierarchy.get("root") {
            walk(root, &mut out);
        }
        out
    }

    fn stored_hierarchy(col: &mut Collection, name: &str) -> Value {
        let did = col.get_deck_id(name).unwrap().expect("deck present");
        col.speedrun_get_hierarchy(&did.0.to_string()).unwrap()
    }

    /// The deck's materialized card ids, sorted, so two snapshots compare by
    /// value (used to assert card identity survives a refresh).
    fn card_ids(col: &mut Collection, did: DeckId) -> Vec<CardId> {
        let mut ids = col
            .search_cards(SearchNode::from_deck_id(did, true), SortMode::NoOrder)
            .unwrap();
        ids.sort();
        ids
    }

    /// The `content` of the first concept in depth-first order (concepts before
    /// children), or empty. Read and write use the same traversal so they agree.
    fn first_concept_content(hierarchy: &Value) -> String {
        fn walk(node: &Value) -> Option<String> {
            if let Some(concept) = node
                .get("concepts")
                .and_then(Value::as_array)
                .and_then(|cs| cs.first())
            {
                return Some(
                    concept
                        .get("content")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                );
            }
            node.get("children")
                .and_then(Value::as_array)
                .and_then(|kids| kids.iter().find_map(walk))
        }
        hierarchy.get("root").and_then(walk).unwrap_or_default()
    }

    /// Overwrite the first concept's `content` in place, leaving ids/structure
    /// (and the concept count) untouched — a stand-in for a content-only rewrite.
    fn set_first_concept_content(hierarchy: &mut Value, content: &str) {
        fn walk(node: &mut Value, content: &str) -> bool {
            if let Some(concept) = node
                .get_mut("concepts")
                .and_then(Value::as_array_mut)
                .and_then(|cs| cs.first_mut())
            {
                concept["content"] = json!(content);
                return true;
            }
            if let Some(kids) = node.get_mut("children").and_then(Value::as_array_mut) {
                return kids.iter_mut().any(|kid| walk(kid, content));
            }
            false
        }
        if let Some(root) = hierarchy.get_mut("root") {
            walk(root, content);
        }
    }

    /// The MCAT bundle's first-concept content, straight from the bundled JSON.
    fn mcat_bundle_first_concept_content() -> String {
        let bundle: Value = serde_json::from_str(BUNDLED_DECKS[1]).unwrap();
        first_concept_content(&bundle)
    }

    #[test]
    fn ensure_seeded_creates_bundled_decks_and_materializes_cards() {
        let mut col = Collection::new();
        let names = deck_names();
        for name in &names {
            assert!(col.get_deck_id(name).unwrap().is_none());
        }

        let result = col.speedrun_ensure_seeded().unwrap();
        assert_eq!(json_names(&result, "seeded"), names);
        assert!(json_names(&result, "updated").is_empty());

        // Each deck now exists and has materialized concept cards.
        for name in &names {
            let did = col.get_deck_id(name).unwrap().expect("deck seeded");
            assert!(card_count(&mut col, did) > 0, "deck {name} has cards");
        }
    }

    #[test]
    fn seeded_mcat_deck_has_all_five_topics_fully_populated() {
        let mut col = Collection::new();
        col.speedrun_ensure_seeded().unwrap();

        let mcat = mcat_name();
        let stored = stored_hierarchy(&mut col, &mcat);
        // The whole bundle: 20 concepts / 80 problems across 5 topic leaves.
        assert_eq!(count_concepts(&stored), 20, "20 concepts total");
        assert_eq!(count_problems(&stored), 80, "80 problems total");

        let leaves = leaf_concept_counts(&stored);
        assert_eq!(leaves.len(), 5, "five topic leaves");
        for (title, n) in &leaves {
            assert_eq!(*n, 4, "topic {title:?} holds 4 concepts");
        }
        // The Physics topics specifically (the regression): each populated.
        for physics in [
            "Kinematics and Dynamics",
            "Work and Energy",
            "Thermodynamics",
        ] {
            assert!(
                leaves.iter().any(|(t, n)| t == physics && *n == 4),
                "physics topic {physics:?} has 4 concepts"
            );
        }

        // One materialized card per concept.
        let did = col.get_deck_id(&mcat).unwrap().unwrap();
        assert_eq!(card_count(&mut col, did), 20);
    }

    #[test]
    fn ensure_seeded_is_idempotent() {
        let mut col = Collection::new();
        assert_eq!(
            col.speedrun_ensure_seeded().unwrap()["seeded"]
                .as_array()
                .unwrap()
                .len(),
            2
        );

        let names = deck_names();
        let before: Vec<(DeckId, usize)> = names
            .iter()
            .map(|name| {
                let did = col.get_deck_id(name).unwrap().unwrap();
                (did, card_count(&mut col, did))
            })
            .collect();

        // A second call changes nothing: no reseed, no refresh, no duplicate
        // deck, no extra cards.
        let again = col.speedrun_ensure_seeded().unwrap();
        assert!(json_names(&again, "seeded").is_empty());
        assert!(json_names(&again, "updated").is_empty());
        for (name, (did, count)) in names.iter().zip(before) {
            assert_eq!(col.get_deck_id(name).unwrap().unwrap(), did);
            assert_eq!(card_count(&mut col, did), count);
        }
    }

    #[test]
    fn ensure_seeded_backfills_a_stale_bundled_deck() {
        let mut col = Collection::new();
        let mcat = mcat_name();

        // Simulate a deck seeded from an older bundle: the MCAT deck exists, but
        // only the Organic Chemistry side is populated (2 concepts) and the
        // Physics leaves are empty — the exact state the bug reported. The two
        // concept ids match the current bundle so their cards must be preserved.
        let stale = json!({
            "deckId": "new",
            "root": { "id": "sr-mcat3135-root", "title": mcat, "concepts": [], "children": [
                { "id": "sr-mcat3135-orgo", "title": "Organic Chemistry", "concepts": [], "children": [
                    { "id": "sr-mcat3135-ch31", "title": "OC topic", "children": [], "concepts": [
                        { "id": "sr-mcat3135-ch31-c1", "title": "a", "content": "a", "problems": [] },
                        { "id": "sr-mcat3135-ch31-c2", "title": "b", "content": "b", "problems": [] },
                    ]},
                ]},
                { "id": "sr-mcat3135-physics", "title": "Physics", "concepts": [], "children": [
                    { "id": "sr-mcat3135-ch33", "title": "Kinematics and Dynamics", "children": [], "concepts": [] },
                ]},
            ]}
        });
        let did = DeckId(
            col.speedrun_save_hierarchy(stale).unwrap()["deckId"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
        );
        col.speedrun_next_card(did).unwrap();
        assert_eq!(
            card_count(&mut col, did),
            2,
            "stale deck starts with 2 cards"
        );

        // ensure_seeded finds no recorded signature for this pre-existing deck,
        // so it refreshes it to the current bundle in place (same deck id); the
        // absent demo deck is freshly created.
        let result = col.speedrun_ensure_seeded().unwrap();
        assert!(
            json_names(&result, "updated").contains(&mcat),
            "MCAT refreshed"
        );
        assert!(
            json_names(&result, "seeded").contains(&deck_names()[0]),
            "demo created"
        );

        // The MCAT deck now carries the full bundle, Physics included, and grew
        // from 2 to 20 cards (the 2 original concept cards were kept, not
        // recreated: reconcile is keyed by conceptId).
        let stored = stored_hierarchy(&mut col, &mcat);
        assert_eq!(count_concepts(&stored), 20);
        for physics in [
            "Kinematics and Dynamics",
            "Work and Energy",
            "Thermodynamics",
        ] {
            assert!(
                leaf_concept_counts(&stored)
                    .iter()
                    .any(|(t, n)| t == physics && *n == 4),
                "physics topic {physics:?} backfilled"
            );
        }
        assert_eq!(
            col.get_deck_id(&mcat).unwrap().unwrap(),
            did,
            "same deck id"
        );
        assert_eq!(card_count(&mut col, did), 20);
    }

    #[test]
    fn ensure_seeded_refreshes_when_bundle_content_changed_same_concept_count() {
        let mut col = Collection::new();
        let mcat = mcat_name();

        // Initial seed: the deck is created from the current bundle and its
        // content signature recorded.
        col.speedrun_ensure_seeded().unwrap();
        let did = col.get_deck_id(&mcat).unwrap().unwrap();
        let cards_before = card_ids(&mut col, did);
        assert_eq!(cards_before.len(), 20, "20 concept cards materialized");

        // Simulate a deck seeded from an OLDER bundle of the same shape: same 20
        // concept ids (so a refresh adds/removes no cards), but different content
        // and a stale recorded signature. count_concepts is unchanged — the bug
        // the concept-count guard missed.
        let mut edited = stored_hierarchy(&mut col, &mcat);
        set_first_concept_content(&mut edited, "OLD STALE CONTENT");
        assert_eq!(count_concepts(&edited), 20, "concept count unchanged");
        col.speedrun_save_hierarchy(edited).unwrap();
        col.record_seed_signature(did, "stale-signature").unwrap();

        // Reopen: the current bundle's signature differs from the stale recorded
        // one, so the deck refreshes to the bundle even though the count matched.
        let result = col.speedrun_ensure_seeded().unwrap();
        assert!(
            json_names(&result, "updated").contains(&mcat),
            "MCAT refreshed on a content-only change"
        );

        // The stored blob is back to the bundle content (the stale edit is gone).
        let refreshed = stored_hierarchy(&mut col, &mcat);
        assert_eq!(count_concepts(&refreshed), 20);
        assert_ne!(first_concept_content(&refreshed), "OLD STALE CONTENT");
        assert_eq!(
            first_concept_content(&refreshed),
            mcat_bundle_first_concept_content(),
            "stored content matches the current bundle"
        );

        // Same deck id, and every concept card kept its identity (same id set) —
        // reconcile is keyed by conceptId — so FSRS progress is preserved.
        assert_eq!(col.get_deck_id(&mcat).unwrap().unwrap(), did, "same deck id");
        assert_eq!(
            card_ids(&mut col, did),
            cards_before,
            "matching-conceptId cards preserved by id (progress kept)"
        );

        // A second reopen with no further bundle change is idempotent.
        let again = col.speedrun_ensure_seeded().unwrap();
        assert!(
            !json_names(&again, "updated").contains(&mcat),
            "no refresh when the bundle is unchanged"
        );
    }

    #[test]
    fn ensure_seeded_leaves_user_edited_deck_when_bundle_unchanged() {
        let mut col = Collection::new();
        let mcat = mcat_name();

        // Seed once: records the current bundle's signature for the deck.
        col.speedrun_ensure_seeded().unwrap();
        let did = col.get_deck_id(&mcat).unwrap().unwrap();

        // The user edits the deck in-app. Authoring saves the blob but does not
        // touch the seed signature.
        let mut edited = stored_hierarchy(&mut col, &mcat);
        set_first_concept_content(&mut edited, "USER EDIT");
        col.speedrun_save_hierarchy(edited).unwrap();

        // Reopen: the bundle is unchanged since the last seed, so the deck is
        // left untouched and the user's edit survives (not clobbered).
        let result = col.speedrun_ensure_seeded().unwrap();
        assert!(
            !json_names(&result, "updated").contains(&mcat),
            "unchanged bundle does not refresh a user-edited deck"
        );
        assert_eq!(
            first_concept_content(&stored_hierarchy(&mut col, &mcat)),
            "USER EDIT",
            "user edit preserved"
        );
        assert_eq!(col.get_deck_id(&mcat).unwrap().unwrap(), did, "same deck id");
    }

    #[test]
    fn ensure_seeded_copies_bundled_media_into_collection() {
        // A filesystem-backed collection (unlike Collection::new) has a media
        // folder, so the bundled images are actually copied.
        let (mut col, _tmp) = open_fs_test_collection("speedrun_media");
        col.speedrun_ensure_seeded().unwrap();

        // Every bundled image lands in the media folder under its exact name.
        for &(name, _) in SEED_MEDIA {
            assert!(
                col.media_folder.join(name).exists(),
                "bundled media {name:?} copied into collection media"
            );
        }
        // Guard the copy path from silently exercising an empty bundle: the
        // media folder ships placeholders (and/or the real figures) at build
        // time, so the table is populated.
        assert!(
            !SEED_MEDIA.is_empty(),
            "seed media bundle is non-empty (placeholders and/or real figures)"
        );
    }

    #[test]
    fn seed_media_copy_is_idempotent_and_overwrites_in_place() {
        let (col, _tmp) = open_fs_test_collection("speedrun_media_replace");
        let media = col.media().unwrap();
        let path = col.media_folder.join("placeholder-x.png");

        media.add_or_replace_file("placeholder-x.png", b"one").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"one");

        // Re-copying identical bytes keeps the exact name (idempotent).
        media.add_or_replace_file("placeholder-x.png", b"one").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"one");

        // New bytes under the same name overwrite in place — no hash-suffixed
        // twin, so `<img src="/placeholder-x.png">` keeps resolving.
        media.add_or_replace_file("placeholder-x.png", b"two").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
        let count = std::fs::read_dir(&col.media_folder).unwrap().count();
        assert_eq!(count, 1, "no duplicate file created on overwrite");
    }
}
