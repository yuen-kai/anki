// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getHierarchy, type Hierarchy, type StudyProgress, studyState } from "../lib";
import type { PageLoad } from "./$types";

// Load the authored hierarchy (card/concept content lives here) and the current
// per-concept mastery map. The orchestrator drives the session from `nextCard`
// after this; the progress map only seeds the learning-block "already seen"
// filter. Mirrors speedrun-dashboard/[deckId]/+page.ts.
export const load = (async ({ params }) => {
    let hierarchy: Hierarchy | null = null;
    let progress: StudyProgress = {};
    let error: string | null = null;
    try {
        hierarchy = await getHierarchy(params.deckId);
        progress = (await studyState(params.deckId)).progress;
    } catch (err) {
        error = err instanceof Error ? err.message : String(err);
    }
    return { deckId: params.deckId, hierarchy, progress, error };
}) satisfies PageLoad;
