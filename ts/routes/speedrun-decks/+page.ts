// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { type DeckSummary, listDecks } from "../speedrun-hierarchy/lib";
import type { PageLoad } from "./$types";

export const load = (async () => {
    let decks: DeckSummary[] = [];
    let error: string | null = null;
    try {
        decks = await listDecks();
    } catch (err) {
        error = err instanceof Error ? err.message : String(err);
    }
    return { decks, error };
}) satisfies PageLoad;
