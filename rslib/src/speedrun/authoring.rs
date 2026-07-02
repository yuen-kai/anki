// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Speedrun authoring store, shared by desktop and mobile.
//!
//! Ported from the desktop-only Python (`pylib/anki/speedrun/authoring.py`) into
//! the shared Rust layer so the Qt desktop app and the AnkiDroid fork drive the
//! same RPCs over the same backend, with no per-platform authoring code.
//!
//! One JSON blob per deck lives under the [`AUTHORING_CONFIG_KEY`] collection
//! config map, keyed by the Anki deck id. A deck row is a real Anki deck (so
//! Study and the To Do count keep working); its authored tree/concepts/problems
//! attach by deck id. Nested nodes are store-only, not Anki subdecks.
//!
//! Shapes (mirrored by the frontend's `speedrun-hierarchy/lib.ts`):
//!
//! ```text
//! Hierarchy { deckId, root: Node }
//! Node      { id, title, children: Node[], concepts: Concept[] }
//! Concept   { id, title, content, problems: Problem[] }
//! Problem   { id, prompt, choices: [c0, c1, c2, c3], correctIndex }  // -1 = unset
//! ```
//!
//! The store owns only structure. Materialization into FSRS-scheduled cards and
//! the per-concept mastery state live in [`crate::speedrun::study`], which reads
//! the same [`AUTHORING_CONFIG_KEY`] blob, so the two interoperate. The RPC
//! surface exchanges JSON so the editor's `lib.ts` is unchanged.

use std::collections::HashMap;

use serde_json::json;
use serde_json::Value;

use crate::decks::tree::get_deck_in_tree;
use crate::prelude::*;

/// Collection-config key holding the authored hierarchy blob per deck. Must
/// match the legacy desktop key (and [`crate::speedrun::study`]'s reader) so an
/// existing collection's authored content is preserved and shared.
const AUTHORING_CONFIG_KEY: &str = "speedrun_authoring";

/// The deckId the create flow sends before its Anki deck exists. Matches the
/// desktop Python and the frontend's `isUnsaved`.
const NEW_DECK_ID: &str = "new";

fn empty_hierarchy(deck_id: &str, title: &str) -> Value {
    json!({
        "deckId": deck_id,
        "root": { "id": "root", "title": title, "children": [], "concepts": [] },
    })
}

/// Parse a numeric deck id, matching the service layer's `speedrun_parse_id`.
fn parse_deck_id(value: &str) -> Result<DeckId> {
    match value.parse::<i64>() {
        Ok(id) => Ok(DeckId(id)),
        Err(_) => invalid_input!("invalid deckId: {value}"),
    }
}

impl Collection {
    fn speedrun_authoring_map(&self) -> HashMap<String, Value> {
        self.get_config_optional(AUTHORING_CONFIG_KEY)
            .unwrap_or_default()
    }

    fn save_speedrun_authoring_map(&mut self, store: HashMap<String, Value>) -> Result<()> {
        // Non-undoable: the authored blob is a bespoke store beside Anki's own
        // undo history, matching the study progress store.
        self.set_config_json(AUTHORING_CONFIG_KEY, &store, false)?;
        Ok(())
    }

    /// Every top-level Anki deck as a row: `{ deckId, name, todo }`, where `todo`
    /// is the deck's new + learn + review due count. This replaces the deck
    /// browser (mirrors the desktop `authoring.list_decks`).
    pub(crate) fn speedrun_list_decks(&mut self) -> Result<Value> {
        let tree = self.deck_tree(Some(TimestampSecs::now()))?;
        let rows: Vec<Value> = tree
            .children
            .iter()
            .map(|child| {
                json!({
                    "deckId": child.deck_id.to_string(),
                    "name": child.name,
                    "todo": child.new_count + child.learn_count + child.review_count,
                })
            })
            .collect();
        Ok(Value::Array(rows))
    }

    /// The stored blob for a deck, or a fresh one seeded with the deck's current
    /// name so the editor always has a root title (mirrors the desktop
    /// `authoring.get_hierarchy`). `deckId` may be `"new"`/`""` (the create
    /// flow), which yields an empty scaffold with a blank title.
    pub(crate) fn speedrun_get_hierarchy(&mut self, deck_id: &str) -> Result<Value> {
        if let Some(blob) = self.speedrun_authoring_map().get(deck_id) {
            return Ok(blob.clone());
        }
        let mut title = String::new();
        if !deck_id.is_empty() && deck_id != NEW_DECK_ID {
            if let Ok(did) = deck_id.parse::<i64>() {
                title = self
                    .get_deck(DeckId(did))?
                    .map(|d| d.human_name())
                    .unwrap_or_default();
            }
        }
        Ok(empty_hierarchy(deck_id, &title))
    }

    /// Persist a deck's authored tree and return the resolved `{ deckId, name }`.
    ///
    /// Creates the backing Anki deck for a new hierarchy (root title = deck name)
    /// or renames it when the root title changed, then writes the blob under the
    /// deck id. A new hierarchy with an empty root title is a no-op: there is
    /// nothing to key the blob on yet, so it returns empty ids (mirrors the
    /// desktop `authoring.save_hierarchy`).
    pub(crate) fn speedrun_save_hierarchy(&mut self, mut hierarchy: Value) -> Result<Value> {
        let deck_id = hierarchy
            .get("deckId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let title = hierarchy
            .get("root")
            .and_then(|root| root.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        let did = if deck_id.is_empty() || deck_id == NEW_DECK_ID {
            if title.is_empty() {
                return Ok(json!({ "deckId": "", "name": "" }));
            }
            self.get_or_create_normal_deck(&title)?.id
        } else {
            let did = parse_deck_id(&deck_id)?;
            if !title.is_empty() {
                if let Some(deck) = self.get_deck(did)? {
                    let current = deck.human_name();
                    if !current.is_empty() && current != title {
                        self.rename_deck(did, &title)?;
                    }
                }
            }
            did
        };

        hierarchy["deckId"] = json!(did.0.to_string());
        let mut store = self.speedrun_authoring_map();
        store.insert(did.0.to_string(), hierarchy);
        self.save_speedrun_authoring_map(store)?;

        // Read the name straight from storage so a just-created or just-renamed
        // deck reports its real name (bypasses the deck cache).
        let name = self
            .storage
            .get_deck(did)?
            .map(|d| d.human_name())
            .unwrap_or(title);
        Ok(json!({ "deckId": did.0.to_string(), "name": name }))
    }

    /// Remove the backing Anki deck and drop its authored blob (mirrors the
    /// desktop `authoring.delete_deck`).
    pub(crate) fn speedrun_delete_deck(&mut self, deck_id: &str) -> Result<()> {
        let did = parse_deck_id(deck_id)?;
        self.remove_decks_and_child_decks(&[did])?;
        let mut store = self.speedrun_authoring_map();
        if store.remove(&did.0.to_string()).is_some() {
            self.save_speedrun_authoring_map(store)?;
        }
        Ok(())
    }

    /// Today's Progress counts for the study screen: the deck's remaining
    /// new/learn/review (from the due tree) and how many cards were studied in it
    /// today (mirrors the desktop `authoring`-adjacent `speedrun_study_summary`).
    pub(crate) fn speedrun_study_summary(&mut self, deck_id: DeckId) -> Result<Value> {
        let tree = self.deck_tree(Some(TimestampSecs::now()))?;
        let Some(node) = get_deck_in_tree(tree, deck_id) else {
            return Ok(json!({
                "deckName": "",
                "new": 0,
                "learn": 0,
                "review": 0,
                "studiedToday": 0,
            }));
        };
        let today = self.counts_for_deck_today(deck_id)?;
        Ok(json!({
            "deckName": node.name,
            "new": node.new_count,
            "learn": node.learn_count,
            "review": node.review_count,
            "studiedToday": today.new + today.review,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored_map(col: &Collection) -> HashMap<String, Value> {
        col.get_config_optional(AUTHORING_CONFIG_KEY)
            .unwrap_or_default()
    }

    #[test]
    fn get_hierarchy_new_deck_is_a_blank_scaffold() {
        let mut col = Collection::new();
        let h = col.speedrun_get_hierarchy("new").unwrap();
        assert_eq!(h["deckId"], json!("new"));
        assert_eq!(h["root"]["title"], json!(""));
        assert_eq!(h["root"]["children"], json!([]));
        assert_eq!(h["root"]["concepts"], json!([]));
    }

    #[test]
    fn save_new_hierarchy_creates_the_backing_deck() {
        let mut col = Collection::new();
        let draft = json!({
            "deckId": "new",
            "root": { "id": "root", "title": "Biochem", "children": [], "concepts": [] },
        });
        let result = col.speedrun_save_hierarchy(draft).unwrap();
        let did = result["deckId"].as_str().unwrap().to_string();
        assert_ne!(did, "");
        assert_ne!(did, "new");
        assert_eq!(result["name"], json!("Biochem"));

        // The deck now exists and the blob round-trips under its real id.
        let did_num: i64 = did.parse().unwrap();
        assert_eq!(
            col.get_deck(DeckId(did_num)).unwrap().unwrap().human_name(),
            "Biochem"
        );
        let reloaded = col.speedrun_get_hierarchy(&did).unwrap();
        assert_eq!(reloaded["deckId"], json!(did));
        assert_eq!(reloaded["root"]["title"], json!("Biochem"));
    }

    #[test]
    fn save_new_hierarchy_with_blank_title_is_a_noop() {
        let mut col = Collection::new();
        let draft = json!({
            "deckId": "new",
            "root": { "id": "root", "title": "   ", "children": [], "concepts": [] },
        });
        let result = col.speedrun_save_hierarchy(draft).unwrap();
        assert_eq!(result, json!({ "deckId": "", "name": "" }));
        assert!(stored_map(&col).is_empty());
    }

    #[test]
    fn save_existing_hierarchy_renames_the_deck_on_title_change() {
        let mut col = Collection::new();
        let did = col.get_or_create_normal_deck("Old name").unwrap().id;
        let blob = json!({
            "deckId": did.0.to_string(),
            "root": { "id": "root", "title": "New name", "children": [], "concepts": [] },
        });
        let result = col.speedrun_save_hierarchy(blob).unwrap();
        assert_eq!(result["deckId"], json!(did.0.to_string()));
        assert_eq!(result["name"], json!("New name"));
        assert_eq!(
            col.storage.get_deck(did).unwrap().unwrap().human_name(),
            "New name"
        );
    }

    #[test]
    fn list_decks_reports_top_level_decks_with_todo_counts() {
        let mut col = Collection::new();
        col.get_or_create_normal_deck("Cardio").unwrap();
        col.get_or_create_normal_deck("Renal").unwrap();
        let decks = col.speedrun_list_decks().unwrap();
        let names: Vec<String> = decks
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["name"].as_str().unwrap().to_string())
            .collect();
        assert!(names.contains(&"Cardio".to_string()));
        assert!(names.contains(&"Renal".to_string()));
        for row in decks.as_array().unwrap() {
            assert!(row["deckId"].is_string());
            assert!(row["todo"].is_number());
        }
    }

    #[test]
    fn delete_deck_removes_the_deck_and_its_blob() {
        let mut col = Collection::new();
        let did = col
            .speedrun_save_hierarchy(json!({
                "deckId": "new",
                "root": { "id": "root", "title": "Doomed", "children": [], "concepts": [] },
            }))
            .unwrap()["deckId"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(stored_map(&col).contains_key(&did));

        col.speedrun_delete_deck(&did).unwrap();
        assert!(!stored_map(&col).contains_key(&did));
        let did_num: i64 = did.parse().unwrap();
        assert!(col.get_deck(DeckId(did_num)).unwrap().is_none());
    }

    #[test]
    fn study_summary_reports_counts_for_a_known_deck_and_zeros_for_unknown() {
        let mut col = Collection::new();
        let did = col.get_or_create_normal_deck("Physio").unwrap().id;
        let summary = col.speedrun_study_summary(did).unwrap();
        assert_eq!(summary["deckName"], json!("Physio"));
        assert_eq!(summary["new"], json!(0));
        assert_eq!(summary["studiedToday"], json!(0));

        // A deck id with no node in the tree reports the empty shape.
        let missing = col.speedrun_study_summary(DeckId(99_999)).unwrap();
        assert_eq!(
            missing,
            json!({ "deckName": "", "new": 0, "learn": 0, "review": 0, "studiedToday": 0 })
        );
    }
}
