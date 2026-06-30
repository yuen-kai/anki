// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { MemoryScore } from "@generated/anki/scheduler_pb";

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
