// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { DeckSummary } from "../speedrun-hierarchy/lib";

// Counts derived from a deck's authored hierarchy + per-concept progress, for
// the home row. `completion` is the share of concepts that have moved past the
// Learn stage (0..1).
export interface DeckMetrics {
    topics: number;
    concepts: number;
    completion: number;
}

// A deck plus its derived counts. `metrics` is null when the per-deck read
// failed, so a single bad deck degrades to "counts unavailable" instead of
// taking down the whole list.
export interface DeckRow extends DeckSummary {
    metrics: DeckMetrics | null;
}
