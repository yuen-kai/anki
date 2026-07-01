// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
import { getMemoryScore, getSpeedrunProgress } from "@generated/backend";

import {
    buildTopicsView,
    envelopeFromMemoryScore,
    type ScoreEnvelope,
    type TopicsView,
} from "../lib";
import type { PageLoad } from "./$types";

export const load = (async ({ params }) => {
    const deckId = BigInt(params.deckId);

    let memory: ScoreEnvelope | null = null;
    let memoryError: string | null = null;
    try {
        const score = await getMemoryScore({ deckId }, { alertOnError: false });
        memory = envelopeFromMemoryScore(score);
    } catch (err) {
        memoryError = err instanceof Error ? err.message : String(err);
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

    return { memory, memoryError, topics, topicsError };
}) satisfies PageLoad;
