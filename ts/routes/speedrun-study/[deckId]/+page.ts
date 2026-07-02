// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { SpeedrunProgress } from "@generated/anki/scheduler_pb";
import {
    getMemoryScore,
    getPerformanceScore,
    getReadinessScore,
    getSpeedrunProgress,
    getSpeedrunScoreBreakdown,
} from "@generated/backend";

import { envelopeFromMemoryScore, envelopeFromScoreEnvelope, type ScoreEnvelope } from "../../speedrun-dashboard/lib";
import { getHierarchy, type Hierarchy } from "../../speedrun-hierarchy/lib";
import {
    buildConceptTree,
    buildSubjectBreakdown,
    isFilteredDeck,
    type StudySummary,
    studySummary,
    type SubjectBreakdown,
} from "../lib";
import type { PageLoad } from "./$types";

const quiet = { alertOnError: false } as const;

const message = (err: unknown): string => (err instanceof Error ? err.message : String(err));

export const load = (async ({ params }) => {
    const deckId = params.deckId;
    const bigDeckId = BigInt(deckId);

    // Each source is isolated: one failed RPC leaves the rest of the screen up.
    let summary: StudySummary | null = null;
    let summaryError: string | null = null;
    try {
        summary = await studySummary(deckId);
    } catch (err) {
        summaryError = message(err);
    }

    let memory: ScoreEnvelope | null = null;
    let memoryError: string | null = null;
    try {
        memory = envelopeFromMemoryScore(await getMemoryScore({ deckId: bigDeckId }, quiet));
    } catch (err) {
        memoryError = message(err);
    }

    let performance: ScoreEnvelope | null = null;
    let performanceError: string | null = null;
    try {
        performance = envelopeFromScoreEnvelope(await getPerformanceScore({ deckId: bigDeckId }, quiet));
    } catch (err) {
        performanceError = message(err);
    }

    let readiness: ScoreEnvelope | null = null;
    let readinessError: string | null = null;
    try {
        readiness = envelopeFromScoreEnvelope(await getReadinessScore({ deckId: bigDeckId }, quiet));
    } catch (err) {
        readinessError = message(err);
    }

    let progress: SpeedrunProgress | null = null;
    let progressError: string | null = null;
    try {
        progress = await getSpeedrunProgress({ did: bigDeckId }, quiet);
    } catch (err) {
        progressError = message(err);
    }

    let hierarchy: Hierarchy | null = null;
    let hierarchyError: string | null = null;
    try {
        hierarchy = await getHierarchy(deckId);
    } catch (err) {
        hierarchyError = message(err);
    }

    let subjects: SubjectBreakdown = buildSubjectBreakdown(null);
    let breakdownError: string | null = null;
    try {
        subjects = buildSubjectBreakdown(await getSpeedrunScoreBreakdown({ did: bigDeckId }, quiet));
    } catch (err) {
        breakdownError = message(err);
    }

    // Filtered detection is best-effort: a failure just hides Rebuild/Empty.
    let filtered = false;
    try {
        filtered = await isFilteredDeck(deckId);
    } catch {
        filtered = false;
    }

    const tree = buildConceptTree(hierarchy, progress);
    const treeError = tree ? null : (hierarchyError ?? progressError);

    return {
        deckId,
        summary,
        summaryError,
        memory,
        memoryError,
        performance,
        performanceError,
        readiness,
        readinessError,
        tree,
        treeError,
        subjects,
        breakdownError,
        filtered,
    };
}) satisfies PageLoad;
