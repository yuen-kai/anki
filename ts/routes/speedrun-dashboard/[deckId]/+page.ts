// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
import { getMemoryScore, getPerformanceScore, getReadinessScore, getSpeedrunProgress } from "@generated/backend";

import {
    buildTopicsView,
    envelopeFromMemoryScore,
    envelopeFromScoreEnvelope,
    type ScoreEnvelope,
    type TopicsView,
} from "../lib";
import type { PageLoad } from "./$types";

export const load = (async ({ params }) => {
    const deckId = BigInt(params.deckId);

    let memory: ScoreEnvelope | null = null;
    let memoryError: string | null = null;
    try {
        memory = envelopeFromMemoryScore(
            await getMemoryScore({ deckId }, { alertOnError: false }),
        );
    } catch (err) {
        memoryError = err instanceof Error ? err.message : String(err);
    }

    let performance: ScoreEnvelope | null = null;
    let performanceError: string | null = null;
    try {
        performance = envelopeFromScoreEnvelope(
            await getPerformanceScore({ deckId }, { alertOnError: false }),
        );
    } catch (err) {
        performanceError = err instanceof Error ? err.message : String(err);
    }

    let readiness: ScoreEnvelope | null = null;
    let readinessError: string | null = null;
    try {
        readiness = envelopeFromScoreEnvelope(
            await getReadinessScore({ deckId }, { alertOnError: false }),
        );
    } catch (err) {
        readinessError = err instanceof Error ? err.message : String(err);
    }

    let topics: TopicsView = buildTopicsView(null);
    let topicsError: string | null = null;
    try {
        topics = buildTopicsView(
            await getSpeedrunProgress({ did: deckId }, { alertOnError: false }),
        );
    } catch (err) {
        topicsError = err instanceof Error ? err.message : String(err);
    }

    return {
        memory,
        memoryError,
        performance,
        performanceError,
        readiness,
        readinessError,
        topics,
        topicsError,
    };
}) satisfies PageLoad;
