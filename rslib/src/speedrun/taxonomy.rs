// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! AAMC MCAT topic taxonomy (seed subset) and pure coverage/weakness helpers.
//!
//! The taxonomy mirrors the AAMC content outline as a three-level tree —
//! foundation → content category → topic — encoded with stable node IDs of the
//! form `mcat::<foundation>::<category>::<topic>` (lowercase snake). A node's
//! `parent_id` is always its own ID minus the last `::` segment; the foundation
//! is the root and has no parent.
//!
//! Only leaf **topics** are scorable: they carry `in_scope = true` and an
//! `exam_weight`. The structural foundation/category nodes carry `in_scope =
//! false` and `exam_weight = 0.0`, so weight lives in exactly one place (the
//! leaf) and the helpers never double-count a branch.
//!
//! ## Seed weights
//!
//! Weights are normalized within the foundation (the eight leaf topics sum to
//! `1.0`) and seeded to approximate AAMC high-yield emphasis: enzyme kinetics
//! and amino-acid / protein structure carry the most, niche metabolism the
//! least. They are plain inline constants for now — easy to read and extend,
//! and flagged tunable. Externalizing them to a shipped data file is deferred
//! until there is an actual tuning workflow (editable taxonomy is post-launch
//! per the taxonomy spec).
//!
//! Everything here is pure: no I/O, no collection access, no protobuf.

use std::collections::HashSet;

/// A node in the MCAT topic tree: a foundation, a content category, or a topic.
///
/// The field shape is a cross-phase contract; later phases (queue, scores,
/// study model) read these fields directly.
#[derive(Debug, Clone, PartialEq)]
pub struct TopicNode {
    /// Stable ID, e.g. `mcat::biomolecules::enzymes::kinetics`.
    pub id: String,
    /// Human-readable label for UI.
    pub label: String,
    /// ID of the parent node; `None` for the foundation (root).
    pub parent_id: Option<String>,
    /// Share of exam emphasis; non-zero only on in-scope leaf topics.
    pub exam_weight: f32,
    /// Whether this node counts toward coverage/weighting (leaf topics only).
    pub in_scope: bool,
}

/// Weight on recent accuracy in [`topic_weakness`]. Tunable; sums to `1.0` with
/// [`W_RET`].
pub const W_ACC: f32 = 0.5;
/// Weight on `1 - retrievability` in [`topic_weakness`]. Tunable; sums to `1.0`
/// with [`W_ACC`].
pub const W_RET: f32 = 0.5;

const FOUNDATION_ID: &str = "mcat::biomolecules";
const FOUNDATION_LABEL: &str = "Biomolecules";

/// The full seed tree: foundation, its content categories, and their topics,
/// with `parent_id` wiring already resolved.
pub fn seed_taxonomy() -> Vec<TopicNode> {
    // Content categories under the foundation, in display order.
    const CATEGORIES: &[(&str, &str)] = &[
        ("amino_acids", "Amino Acids"),
        ("proteins", "Proteins"),
        ("enzymes", "Enzymes"),
    ];
    // Leaf topics: (category, topic, label, exam_weight). Weights sum to 1.0.
    const TOPICS: &[(&str, &str, &str, f32)] = &[
        ("amino_acids", "structure", "Structure", 0.15),
        ("amino_acids", "pka_titration", "pKa & Titration", 0.12),
        ("amino_acids", "metabolism", "Metabolism", 0.08),
        ("proteins", "structure_levels", "Levels of Structure", 0.15),
        ("proteins", "folding", "Folding", 0.10),
        ("enzymes", "kinetics", "Kinetics", 0.18),
        ("enzymes", "inhibition", "Inhibition", 0.12),
        ("enzymes", "regulation", "Regulation", 0.10),
    ];

    let mut nodes = Vec::with_capacity(1 + CATEGORIES.len() + TOPICS.len());

    // Foundation (root): structural, so no weight or scope.
    nodes.push(TopicNode {
        id: FOUNDATION_ID.to_string(),
        label: FOUNDATION_LABEL.to_string(),
        parent_id: None,
        exam_weight: 0.0,
        in_scope: false,
    });

    // Content categories: structural children of the foundation.
    for (seg, label) in CATEGORIES {
        nodes.push(TopicNode {
            id: format!("{FOUNDATION_ID}::{seg}"),
            label: (*label).to_string(),
            parent_id: Some(FOUNDATION_ID.to_string()),
            exam_weight: 0.0,
            in_scope: false,
        });
    }

    // Leaf topics: the scorable, weighted nodes.
    for (category, topic, label, weight) in TOPICS {
        nodes.push(TopicNode {
            id: format!("{FOUNDATION_ID}::{category}::{topic}"),
            label: (*label).to_string(),
            parent_id: Some(format!("{FOUNDATION_ID}::{category}")),
            exam_weight: *weight,
            in_scope: true,
        });
    }

    nodes
}

/// Node-based coverage: the fraction of distinct in-scope topics that have at
/// least one covering entry.
///
/// This is intentionally **not** card-count based — a deck with thousands of
/// cards spread over only a few topics still reads as low coverage. Both inputs
/// are treated as sets, so duplicates and out-of-scope entries don't skew it.
/// Returns `0.0` when there are no in-scope topics.
pub fn coverage_pct(in_scope_topic_ids: &[String], covered_topic_ids: &[String]) -> f32 {
    let in_scope: HashSet<&str> = in_scope_topic_ids.iter().map(String::as_str).collect();
    if in_scope.is_empty() {
        return 0.0;
    }
    let covered: HashSet<&str> = covered_topic_ids.iter().map(String::as_str).collect();
    in_scope.intersection(&covered).count() as f32 / in_scope.len() as f32
}

/// Sum of `exam_weight` over the in-scope topics that are covered.
///
/// Structural nodes (`in_scope == false`) never contribute, so passing a
/// foundation or category ID is a no-op. With the seed tree, covering every
/// topic yields `1.0`.
pub fn weighted_coverage(nodes: &[TopicNode], covered_topic_ids: &[String]) -> f32 {
    let covered: HashSet<&str> = covered_topic_ids.iter().map(String::as_str).collect();
    nodes
        .iter()
        .filter(|n| n.in_scope && covered.contains(n.id.as_str()))
        .map(|n| n.exam_weight)
        .sum()
}

/// Wednesday weakness proxy for a topic, in `[0, 1]` (higher = weaker).
///
/// Blends miss rate (`1 - recent_accuracy`) and forgetting (`1 -
/// mean_retrievability`) by [`W_ACC`] / [`W_RET`]. Inputs outside `[0, 1]` are
/// tolerated; the result is clamped. This is the single definition consumed by
/// the topic queue and study model in later phases.
pub fn topic_weakness(recent_accuracy: f32, mean_retrievability: f32) -> f32 {
    (W_ACC * (1.0 - recent_accuracy) + W_RET * (1.0 - mean_retrievability)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The leaf topic IDs other Phase-1 work references by exact string.
    const EXPECTED_TOPIC_IDS: &[&str] = &[
        "mcat::biomolecules::amino_acids::structure",
        "mcat::biomolecules::amino_acids::pka_titration",
        "mcat::biomolecules::amino_acids::metabolism",
        "mcat::biomolecules::proteins::structure_levels",
        "mcat::biomolecules::proteins::folding",
        "mcat::biomolecules::enzymes::kinetics",
        "mcat::biomolecules::enzymes::inhibition",
        "mcat::biomolecules::enzymes::regulation",
    ];

    fn id_is_well_formed(id: &str) -> bool {
        !id.is_empty()
            && id.split("::").all(|seg| {
                !seg.is_empty()
                    && seg
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            })
    }

    fn topic_ids() -> Vec<String> {
        EXPECTED_TOPIC_IDS.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn seed_integrity() {
        let nodes = seed_taxonomy();
        let ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();

        // 1 foundation + 3 categories + 8 topics.
        assert_eq!(nodes.len(), 12);

        // Foundation is present and is the root.
        let foundation = nodes
            .iter()
            .find(|n| n.id == FOUNDATION_ID)
            .expect("foundation present");
        assert_eq!(foundation.parent_id, None);
        assert!(!foundation.in_scope);

        // The three content categories hang off the foundation.
        for cat in ["amino_acids", "proteins", "enzymes"] {
            let cat_id = format!("{FOUNDATION_ID}::{cat}");
            let node = nodes
                .iter()
                .find(|n| n.id == cat_id)
                .unwrap_or_else(|| panic!("category {cat_id} present"));
            assert_eq!(node.parent_id.as_deref(), Some(FOUNDATION_ID));
            assert!(!node.in_scope, "categories are structural");
        }

        // Every contract topic ID is present, well-formed, in scope, weighted,
        // and wired to an existing category parent.
        for &tid in EXPECTED_TOPIC_IDS {
            let node = nodes
                .iter()
                .find(|n| n.id == tid)
                .unwrap_or_else(|| panic!("topic {tid} present"));
            assert!(id_is_well_formed(tid), "{tid} should be well-formed");
            assert!(node.in_scope, "{tid} should be in scope");
            assert!(node.exam_weight > 0.0, "{tid} should have a weight");
            let parent = node.parent_id.as_deref().expect("topic has a parent");
            assert!(ids.contains(parent), "{tid} parent {parent} should exist");
        }

        // Generic wiring invariants for every node.
        for n in &nodes {
            assert!(id_is_well_formed(&n.id), "{} should be well-formed", n.id);
            if let Some(parent) = &n.parent_id {
                assert_ne!(parent, &n.id, "no node is its own parent");
                assert!(
                    ids.contains(parent.as_str()),
                    "parent {parent} should exist"
                );
                // parent_id == id minus its last segment.
                let derived = n.id.rsplit_once("::").map(|(head, _)| head);
                assert_eq!(derived, Some(parent.as_str()), "{} parent wiring", n.id);
            }
        }

        // Leaf weights are normalized within the foundation.
        let total: f32 = nodes
            .iter()
            .filter(|n| n.in_scope)
            .map(|n| n.exam_weight)
            .sum();
        assert!(
            (total - 1.0).abs() < 1e-5,
            "weights should sum to 1.0, got {total}"
        );
    }

    #[test]
    fn coverage_pct_is_node_based() {
        let in_scope = topic_ids();

        // Everything covered → 100%.
        assert!((coverage_pct(&in_scope, &in_scope) - 1.0).abs() < 1e-6);

        // Empty in-scope set → 0.0, not NaN.
        assert_eq!(coverage_pct(&[], &in_scope), 0.0);

        // Half covered (4 of 8).
        let half: Vec<String> = in_scope.iter().take(4).cloned().collect();
        assert!((coverage_pct(&in_scope, &half) - 0.5).abs() < 1e-6);

        // Lopsided: a huge "deck" whose 15,000 card entries map to just 3 of
        // 100 in-scope topics. Node-based coverage is 3%, proving card volume
        // doesn't inflate it.
        let many_topics: Vec<String> = (0..100)
            .map(|i| format!("mcat::biomolecules::synthetic::t{i}"))
            .collect();
        let mut many_cards: Vec<String> = Vec::new();
        for _ in 0..5000 {
            for i in 0..3 {
                many_cards.push(format!("mcat::biomolecules::synthetic::t{i}"));
            }
        }
        assert_eq!(many_cards.len(), 15_000);
        let pct = coverage_pct(&many_topics, &many_cards);
        assert!(
            (pct - 0.03).abs() < 1e-6,
            "lopsided coverage should be 3%, got {pct}"
        );
    }

    #[test]
    fn weighted_coverage_sums_topic_weights() {
        let nodes = seed_taxonomy();

        // Covering every topic sums to the normalized total.
        assert!((weighted_coverage(&nodes, &topic_ids()) - 1.0).abs() < 1e-5);

        // Nothing covered → 0.0.
        assert_eq!(weighted_coverage(&nodes, &[]), 0.0);

        // A single covered topic contributes exactly its own weight.
        let kinetics = "mcat::biomolecules::enzymes::kinetics".to_string();
        let expected = nodes.iter().find(|n| n.id == kinetics).unwrap().exam_weight;
        let got = weighted_coverage(&nodes, std::slice::from_ref(&kinetics));
        assert!(
            (got - expected).abs() < 1e-6,
            "expected {expected}, got {got}"
        );

        // Structural nodes never contribute, even if passed in.
        let structural = vec![
            FOUNDATION_ID.to_string(),
            format!("{FOUNDATION_ID}::enzymes"),
        ];
        assert_eq!(weighted_coverage(&nodes, &structural), 0.0);
    }

    #[test]
    fn topic_weakness_bounds_and_midpoint() {
        // Constants are a normalized split.
        assert!((W_ACC + W_RET - 1.0).abs() < 1e-6);

        // Perfect recall and retrievability → no weakness.
        assert_eq!(topic_weakness(1.0, 1.0), 0.0);
        // Worst case → maximal weakness.
        assert_eq!(topic_weakness(0.0, 0.0), 1.0);
        // Balanced midpoint.
        assert!((topic_weakness(0.5, 0.5) - 0.5).abs() < 1e-6);
        // Mixed: 0.5*(1-0.8) + 0.5*(1-0.6) = 0.3.
        assert!((topic_weakness(0.8, 0.6) - 0.3).abs() < 1e-6);

        // Out-of-range inputs are clamped into [0, 1].
        assert_eq!(topic_weakness(-1.0, -1.0), 1.0);
        assert_eq!(topic_weakness(2.0, 2.0), 0.0);
    }
}
