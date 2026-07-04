// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Speedrun `SchedulerService` RPC bodies, split out of the service god file.
//!
//! The typed score RPCs (Memory/Performance/Readiness/breakdown) and the
//! JSON-exchanging study + authoring RPCs live here as free functions taking
//! `&mut Collection`; the trait methods in [`super`] are thin delegators. The
//! engine itself lives in [`crate::speedrun`]; this layer only marshals the
//! protobuf/JSON request and response shapes.

use anki_proto::generic;
use anki_proto::scheduler;
use serde::Deserialize;
use serde_json::Value;

use crate::prelude::*;
use crate::speedrun::parse_id;

/// Request body for the deck-scoped Speedrun study/authoring RPCs
/// (`{ deckId }`). The screens send ids as strings, matching the authoring
/// store; `deckId` may be `"new"`/`""` for the authoring create flow.
#[derive(Deserialize)]
struct DeckRequest {
    #[serde(rename = "deckId")]
    deck_id: String,
}

/// Request body for `SpeedrunAnswerCard`.
#[derive(Deserialize)]
struct AnswerRequest {
    #[serde(rename = "deckId")]
    deck_id: String,
    #[serde(rename = "cardId")]
    card_id: String,
    #[serde(rename = "conceptId")]
    concept_id: String,
    rating: i32,
}

/// Request body for `SpeedrunRecordLearned`.
#[derive(Deserialize)]
struct LearnedRequest {
    #[serde(rename = "deckId")]
    deck_id: String,
    #[serde(rename = "conceptIds", default)]
    concept_ids: Vec<String>,
}

fn json_reply(value: &Value) -> Result<generic::Json> {
    Ok(generic::Json {
        json: serde_json::to_vec(value)?,
    })
}

/// Map the shared Rust [`crate::speedrun::scores::ScoreEnvelope`] (Performance
/// / Readiness) onto its protobuf form.
fn score_envelope_to_proto(
    score: crate::speedrun::scores::ScoreEnvelope,
) -> scheduler::ScoreEnvelope {
    scheduler::ScoreEnvelope {
        estimate: score.estimate,
        range_low: score.range_low,
        range_high: score.range_high,
        coverage_pct: score.coverage_pct,
        confidence: score.confidence.as_str().to_string(),
        updated_at_secs: score.updated_at_secs,
        reasons: score.reasons,
        abstained: score.abstained,
        abstain_reason: score.abstain_reason,
        graded_reviews: score.graded_reviews,
        format: score.format.as_str().to_string(),
    }
}

// --- typed score RPCs ------------------------------------------------------

pub(super) fn memory_score(
    col: &mut Collection,
    input: scheduler::GetMemoryScoreRequest,
) -> Result<scheduler::MemoryScore> {
    let score = col.get_memory_score(input.deck_id.into())?;
    Ok(scheduler::MemoryScore {
        estimate: score.estimate,
        range_low: score.range_low,
        range_high: score.range_high,
        coverage_pct: score.coverage_pct,
        confidence: score.confidence.as_str().to_string(),
        updated_at_secs: score.updated_at_secs,
        reasons: score.reasons,
        abstained: score.abstained,
        abstain_reason: score.abstain_reason,
        graded_reviews: score.graded_reviews,
    })
}

pub(super) fn performance_score(
    col: &mut Collection,
    input: scheduler::GetSpeedrunScoreRequest,
) -> Result<scheduler::ScoreEnvelope> {
    Ok(score_envelope_to_proto(
        col.get_performance_score(input.deck_id.into())?,
    ))
}

pub(super) fn readiness_score(
    col: &mut Collection,
    input: scheduler::GetSpeedrunScoreRequest,
) -> Result<scheduler::ScoreEnvelope> {
    Ok(score_envelope_to_proto(
        col.get_readiness_score(input.deck_id.into())?,
    ))
}

pub(super) fn score_breakdown(
    col: &mut Collection,
    input: anki_proto::decks::DeckId,
) -> Result<scheduler::SpeedrunScoreBreakdown> {
    let topics = col
        .get_speedrun_score_breakdown(input.did.into())?
        .into_iter()
        .map(|stat| scheduler::speedrun_score_breakdown::TopicStat {
            topic_id: stat.topic_id,
            path: stat.path,
            mean_retrievability: stat.mean_retrievability,
            application_accuracy: stat.application_accuracy,
            application_attempts: stat.application_attempts,
            memory_reviews: stat.memory_reviews,
            exam_weight: stat.exam_weight,
            has_application_data: stat.has_application_data,
        })
        .collect();
    Ok(scheduler::SpeedrunScoreBreakdown { topics })
}

// --- JSON study RPCs -------------------------------------------------------

pub(super) fn study_state(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let req: DeckRequest = serde_json::from_slice(&input.json)?;
    let deck_id = DeckId(parse_id(&req.deck_id, "deckId")?);
    json_reply(&col.speedrun_study_state(deck_id)?)
}

pub(super) fn next_card(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let req: DeckRequest = serde_json::from_slice(&input.json)?;
    let deck_id = DeckId(parse_id(&req.deck_id, "deckId")?);
    json_reply(&col.speedrun_next_card(deck_id)?)
}

pub(super) fn answer_card(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let req: AnswerRequest = serde_json::from_slice(&input.json)?;
    let deck_id = DeckId(parse_id(&req.deck_id, "deckId")?);
    let card_id = CardId(parse_id(&req.card_id, "cardId")?);
    json_reply(&col.speedrun_answer_card(deck_id, card_id, &req.concept_id, req.rating)?)
}

pub(super) fn record_learned(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let req: LearnedRequest = serde_json::from_slice(&input.json)?;
    let deck_id = DeckId(parse_id(&req.deck_id, "deckId")?);
    json_reply(&col.speedrun_record_learned(deck_id, &req.concept_ids)?)
}

pub(super) fn study_hierarchy(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let req: DeckRequest = serde_json::from_slice(&input.json)?;
    let deck_id = DeckId(parse_id(&req.deck_id, "deckId")?);
    json_reply(&col.speedrun_study_hierarchy(deck_id)?)
}

// --- JSON authoring RPCs ---------------------------------------------------

pub(super) fn list_decks(col: &mut Collection, _input: generic::Json) -> Result<generic::Json> {
    json_reply(&col.speedrun_list_decks()?)
}

pub(super) fn get_hierarchy(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    // The create flow sends deckId "new"/"", so the id stays a string here.
    let req: DeckRequest = serde_json::from_slice(&input.json)?;
    json_reply(&col.speedrun_get_hierarchy(&req.deck_id)?)
}

pub(super) fn save_hierarchy(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    // The request body is the Hierarchy blob itself.
    let hierarchy: Value = serde_json::from_slice(&input.json)?;
    json_reply(&col.speedrun_save_hierarchy(hierarchy)?)
}

pub(super) fn delete_deck(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let req: DeckRequest = serde_json::from_slice(&input.json)?;
    col.speedrun_delete_deck(&req.deck_id)?;
    json_reply(&serde_json::json!({}))
}

pub(super) fn study_summary(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let req: DeckRequest = serde_json::from_slice(&input.json)?;
    let deck_id = DeckId(parse_id(&req.deck_id, "deckId")?);
    json_reply(&col.speedrun_study_summary(deck_id)?)
}

pub(super) fn ensure_seeded(col: &mut Collection, _input: generic::Json) -> Result<generic::Json> {
    json_reply(&col.speedrun_ensure_seeded()?)
}

// --- AI deck-import RPCs ----------------------------------------------------

pub(super) fn ai_config(col: &mut Collection, _input: generic::Json) -> Result<generic::Json> {
    json_reply(&col.speedrun_ai_config()?)
}

pub(super) fn ai_import(col: &mut Collection, input: generic::Json) -> Result<generic::Json> {
    let request: Value = serde_json::from_slice(&input.json)?;
    json_reply(&col.speedrun_ai_import(request)?)
}
