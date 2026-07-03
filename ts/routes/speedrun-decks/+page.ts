// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { stageIndex } from "../speedrun-dashboard/lib";
import { type DeckSummary, getHierarchy, listDecks, type Node } from "../speedrun-hierarchy/lib";
import { type StudyProgress, studyState } from "../speedrun-review/lib";
import type { DeckMetrics, DeckRow } from "./lib";
import type { PageLoad } from "./$types";

// Walk the tree once: topics are leaf nodes (no children); the bare root of an
// empty deck is "no hierarchy yet", not a topic. Concepts are collected from
// every node (only leaves hold them in practice, but this stays robust if the
// author left some on a branch).
function deckMetrics(root: Node, progress: StudyProgress): DeckMetrics {
    let topics = 0;
    const conceptIds: string[] = [];
    const visit = (node: Node, isRoot: boolean): void => {
        for (const concept of node.concepts) {
            conceptIds.push(concept.id);
        }
        if (node.children.length === 0) {
            if (!isRoot) {
                topics += 1;
            }
            return;
        }
        for (const child of node.children) {
            visit(child, false);
        }
    };
    visit(root, true);

    const concepts = conceptIds.length;
    const pastLearn = conceptIds.filter((id) => {
        const p = progress[id];
        return p ? stageIndex(p.state) >= 1 : false;
    }).length;
    return {
        topics,
        concepts,
        completion: concepts === 0 ? 0 : pastLearn / concepts,
    };
}

async function enrich(deck: DeckSummary): Promise<DeckRow> {
    try {
        const [hierarchy, state] = await Promise.all([
            getHierarchy(deck.deckId),
            studyState(deck.deckId),
        ]);
        return { ...deck, metrics: deckMetrics(hierarchy.root, state.progress) };
    } catch {
        return { ...deck, metrics: null };
    }
}

export const load = (async () => {
    let decks: DeckRow[] = [];
    let error: string | null = null;
    try {
        // Enrich every deck in parallel; each enrich() swallows its own failure.
        decks = await Promise.all((await listDecks()).map(enrich));
    } catch (err) {
        error = err instanceof Error ? err.message : String(err);
    }
    return { decks, error };
}) satisfies PageLoad;
