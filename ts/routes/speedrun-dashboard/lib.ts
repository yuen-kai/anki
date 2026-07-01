// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { MemoryScore, SpeedrunProgress } from "@generated/anki/scheduler_pb";

// The Speedrun evidence envelope (spec-scores §4): the single shape every score
// is rendered as. There is no bare-number path; when `abstained` is true the
// number is withheld and `abstainReason` says what is missing.
export interface ScoreEnvelope {
    estimate: number;
    rangeLow: number;
    rangeHigh: number;
    coveragePct: number;
    confidence: string;
    updatedAtSecs: number;
    reasons: string[];
    abstained: boolean;
    abstainReason: string;
    gradedReviews: number;
}

export function envelopeFromMemoryScore(score: MemoryScore): ScoreEnvelope {
    return {
        estimate: score.estimate,
        rangeLow: score.rangeLow,
        rangeHigh: score.rangeHigh,
        coveragePct: score.coveragePct,
        confidence: score.confidence,
        // updated_at is an int64 (bigint over the wire); seconds fit a number.
        updatedAtSecs: Number(score.updatedAtSecs),
        reasons: score.reasons,
        abstained: score.abstained,
        abstainReason: score.abstainReason,
        gradedReviews: score.gradedReviews,
    };
}

// Performance and Readiness have no backend yet (PRD §6, spec-scores §7): their
// tiles abstain by construction and explain when the score will exist.
export function deferredEnvelope(abstainReason: string): ScoreEnvelope {
    return {
        estimate: 0,
        rangeLow: 0,
        rangeHigh: 0,
        coveragePct: 0,
        confidence: "",
        updatedAtSecs: 0,
        reasons: [],
        abstained: true,
        abstainReason,
        gradedReviews: 0,
    };
}

export function formatEstimate(value: number): string {
    return value.toFixed(2);
}

export function formatRange(low: number, high: number): string {
    return `${low.toFixed(2)} to ${high.toFixed(2)}`;
}

export function formatCoverage(fraction: number): string {
    return `${Math.round(fraction * 100)}%`;
}

export function formatUpdated(secs: number): string {
    if (!secs) {
        return "";
    }
    return new Date(secs * 1000).toLocaleString();
}

export function capitalize(text: string): string {
    return text ? text[0].toUpperCase() + text.slice(1) : text;
}

// The Rust score lists coverage as its first reason, and we already show
// coverage as its own figure, so drop it here to avoid printing it twice.
export function driverReasons(reasons: string[]): string[] {
    return reasons.filter((reason) => !reason.toLowerCase().startsWith("coverage"));
}

// The four-stage mastery ladder (spec-mastery-progression §2), in order. The
// internal "hierarchy" state renders as "Applying": the learner is applying the
// concept with the scaffold, and "hierarchy" names the mechanism, not a stage a
// student would recognize. The blurb is what the learner is doing at that stage.
export const MASTERY_STAGES = [
    { state: "learning", label: "Learning", blurb: "Meeting the idea through contrasting cases" },
    { state: "practicing", label: "Practicing", blurb: "Recalling the idea from a single cue" },
    { state: "hierarchy", label: "Applying", blurb: "Working problems with the principle scaffold" },
    { state: "mastering", label: "Mastering", blurb: "Working problems with the scaffold gone" },
] as const;

export const STAGE_COUNT = MASTERY_STAGES.length;

export function stageIndex(state: string): number {
    const i = MASTERY_STAGES.findIndex((stage) => stage.state === state);
    // An unknown/absent state is the fail-safe first rung (matches the engine's
    // default of `learning`), never a negative index.
    return i < 0 ? 0 : i;
}

export interface TopicRow {
    id: string;
    label: string;
    stage: number;
    stageLabel: string;
}

export interface TopicGroup {
    heading: string;
    topics: TopicRow[];
}

export interface TopicsView {
    groups: TopicGroup[];
    // Count of topics at each rung, index-aligned with MASTERY_STAGES.
    distribution: number[];
    total: number;
}

// Shape the flat per-topic progress into the topics view: rows grouped under
// their hierarchy path (foundation › category), plus the stage distribution for
// the summary. Labels come from the taxonomy path the backend sends, so nothing
// here re-spells a topic; a topic missing its path falls back to its id.
export function buildTopicsView(progress: SpeedrunProgress | null): TopicsView {
    const distribution = new Array(STAGE_COUNT).fill(0);
    const byHeading = new Map<string, TopicRow[]>();

    for (const topic of progress?.topics ?? []) {
        const path = topic.path ?? [];
        const label = path.length ? path[path.length - 1] : topic.topicId;
        const heading = path.length > 1 ? path.slice(0, -1).join(" › ") : "Other topics";
        const stage = stageIndex(topic.state);
        distribution[stage] += 1;

        const row: TopicRow = {
            id: topic.topicId,
            label,
            stage,
            stageLabel: MASTERY_STAGES[stage].label,
        };
        const rows = byHeading.get(heading);
        if (rows) {
            rows.push(row);
        } else {
            byHeading.set(heading, [row]);
        }
    }

    const groups: TopicGroup[] = [...byHeading.entries()]
        .map(([heading, topics]) => ({
            heading,
            topics: topics.sort((a, b) => a.label.localeCompare(b.label)),
        }))
        .sort((a, b) => a.heading.localeCompare(b.heading));

    const total = groups.reduce((sum, group) => sum + group.topics.length, 0);
    return { groups, distribution, total };
}
