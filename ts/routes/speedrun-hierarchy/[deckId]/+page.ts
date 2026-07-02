// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { getHierarchy, type Hierarchy } from "../lib";
import type { PageLoad } from "./$types";

// `deckId` may be "new": the backend then returns an empty scaffold to author
// against, and the real id is minted on the first save.
export const load = (async ({ params }) => {
    let hierarchy: Hierarchy | null = null;
    let error: string | null = null;
    try {
        hierarchy = await getHierarchy(params.deckId);
    } catch (err) {
        error = err instanceof Error ? err.message : String(err);
    }
    return { hierarchy, error };
}) satisfies PageLoad;
