// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type {
    SpeedrunProgress,
    SpeedrunScoreBreakdown,
    SpeedrunScoreBreakdown_TopicStat,
} from "@generated/anki/scheduler_pb";
import {
    getDeckNames,
    speedrunOverviewAction,
    speedrunShowDecks,
    speedrunStartStudy,
    speedrunStudySummary,
} from "@generated/backend";

import { gaugePercent, MASTERY_STAGES, type ScoreEnvelope, STAGE_COUNT, stageIndex } from "../speedrun-dashboard/lib";
import type { Hierarchy, Node } from "../speedrun-hierarchy/lib";

const enc = (value: unknown): Uint8Array => new TextEncoder().encode(JSON.stringify(value));

const dec = <T>(reply: { json: Uint8Array }): T => JSON.parse(new TextDecoder().decode(reply.json)) as T;

// The JSON handlers alert on error by default; the loader isolates every call
// itself, so we opt out of the global dialog (mirrors speedrun-hierarchy/lib).
const quiet = { alertOnError: false } as const;

// ---------------------------------------------------------------------------
// JSON RPC helpers (the study-screen POST handlers)
// ---------------------------------------------------------------------------

// Today's Progress counts for a deck. `deckName` may be "Parent::Child";
// `studiedToday` is today's new + review reps (excludes intraday learning), so
// today's total is studiedToday + new + learn + review and remaining is
// new + learn + review.
export interface StudySummary {
    deckName: string;
    new: number;
    learn: number;
    review: number;
    studiedToday: number;
}

export async function studySummary(deckId: string): Promise<StudySummary> {
    return dec<StudySummary>(await speedrunStudySummary({ json: enc({ deckId }) }, quiet));
}

// Arm the topic-grouped queue and hand off to the reviewer (Qt side effect).
export async function startStudy(deckId: string): Promise<void> {
    await speedrunStartStudy({ json: enc({ deckId }) }, quiet);
}

// The secondary deck-overview actions, preserved in the overflow menu.
export type OverviewAction = "options" | "customStudy" | "unbury" | "description" | "rebuild" | "empty";

export async function overviewAction(action: OverviewAction): Promise<void> {
    await speedrunOverviewAction({ json: enc({ action }) }, quiet);
}

// Back to the Decks home.
export async function showDecks(): Promise<void> {
    await speedrunShowDecks({ json: enc({}) }, quiet);
}

// Whether a deck is a filtered (dynamic) deck, so the overflow can offer
// Rebuild/Empty only where they apply. Phase 1's summary carries no such flag,
// so we derive it from get_deck_names: a filtered deck is present with filtered
// included but absent with it excluded.
export async function isFilteredDeck(deckId: string): Promise<boolean> {
    const id = BigInt(deckId);
    const [all, plain] = await Promise.all([
        getDeckNames({ skipEmptyDefault: false, includeFiltered: true }, quiet),
        getDeckNames({ skipEmptyDefault: false, includeFiltered: false }, quiet),
    ]);
    const has = (names: { entries: { id: bigint }[] }): boolean => names.entries.some((e) => e.id === id);
    return has(all) && !has(plain);
}

// ---------------------------------------------------------------------------
// Today's Progress
// ---------------------------------------------------------------------------

export interface TodayProgress {
    // Reps already done today (studiedToday).
    done: number;
    // Cards still due today (new + learn + review); the headline number.
    remaining: number;
    // done + remaining.
    total: number;
    // done / total, clamped to [0, 1]; 1 when nothing remains.
    fraction: number;
    // Nothing due right now, so the screen shows an "all done today" state.
    allDone: boolean;
}

export function todayProgress(summary: StudySummary | null): TodayProgress {
    const done = summary?.studiedToday ?? 0;
    const remaining = (summary?.new ?? 0) + (summary?.learn ?? 0) + (summary?.review ?? 0);
    const total = done + remaining;
    const fraction = total > 0 ? Math.max(0, Math.min(1, done / total)) : 1;
    return { done, remaining, total, fraction, allDone: remaining === 0 };
}

// The deck's own name for the header: the leaf of a "Parent::Child" path.
export function deckLeafName(deckName: string): string {
    const parts = deckName.split("::");
    return parts[parts.length - 1] || deckName;
}

// The parent path of a "Parent::Child" name, or "" for a top-level deck.
export function deckParentName(deckName: string): string {
    const parts = deckName.split("::");
    return parts.length > 1 ? parts.slice(0, -1).join(" / ") : "";
}

// ---------------------------------------------------------------------------
// Score rings
// ---------------------------------------------------------------------------

// A ring's fill fraction in [0, 1]: where the estimate sits on its own scale
// (ratio scores over 0-1, points scores across 472-528), reusing the linear
// gauge math so the ring and the dashboard gauge always agree.
export function ringFraction(envelope: ScoreEnvelope): number {
    return gaugePercent(envelope.estimate, envelope.format) / 100;
}

// stroke-dasharray geometry for an SVG ring of the given radius: the full
// circumference plus the dash offset that leaves only `fraction` of it drawn.
export function ringDash(fraction: number, radius: number): { circumference: number; offset: number } {
    const circumference = 2 * Math.PI * radius;
    const clamped = Math.max(0, Math.min(1, fraction));
    return { circumference, offset: circumference * (1 - clamped) };
}

// ---------------------------------------------------------------------------
// Concept tree
// ---------------------------------------------------------------------------

export const NOT_STARTED_LABEL = "Not started";

export interface ConceptTreeNode {
    id: string;
    title: string;
    children: ConceptTreeNode[];
    isLeaf: boolean;
    // Leaf mastery stage index (0-3), or null when the leaf maps to no tracked
    // topic ("not started"). Branches carry null.
    stage: number | null;
    // "Learning".."Mastering" for a mapped leaf, "Not started" for an unmapped
    // one, "" for a branch.
    stageLabel: string;
    // 0-1 completion for the node's meter: a leaf from its own stage, a branch
    // from the mean of its leaf-descendant fractions.
    fraction: number;
    // Leaf descendants under this node (1 for a leaf); drives the branch summary.
    leafCount: number;
}

// A mapped leaf's completion: mastering fills the bar, learning a quarter, an
// unmapped leaf nothing. Aligns with the four-pip meter (stage + 1 pips).
function leafFraction(stage: number | null): number {
    return stage === null ? 0 : (stage + 1) / STAGE_COUNT;
}

function makeLeaf(id: string, title: string, stage: number | null): ConceptTreeNode {
    return {
        id,
        title: title || "Untitled",
        children: [],
        isLeaf: true,
        stage,
        stageLabel: stage === null ? NOT_STARTED_LABEL : MASTERY_STAGES[stage].label,
        fraction: leafFraction(stage),
        leafCount: 1,
    };
}

function makeBranch(id: string, title: string, children: ConceptTreeNode[]): ConceptTreeNode {
    const leafCount = children.reduce((n, c) => n + c.leafCount, 0);
    // Mean of leaf-descendant fractions == leaf-count-weighted mean of the
    // children's own fractions.
    const weighted = children.reduce((s, c) => s + c.fraction * c.leafCount, 0);
    return {
        id,
        title: title || "Untitled",
        children,
        isLeaf: false,
        stage: null,
        stageLabel: "",
        fraction: leafCount > 0 ? weighted / leafCount : 0,
        leafCount,
    };
}

// Stage by leaf label (case-insensitive), for overlaying taxonomy progress on
// authored leaves. First mapping wins on a collision.
function stagesByLeafLabel(progress: SpeedrunProgress | null): Map<string, number> {
    const map = new Map<string, number>();
    for (const topic of progress?.topics ?? []) {
        const path = topic.path ?? [];
        const leaf = (path.length ? path[path.length - 1] : topic.topicId).trim().toLowerCase();
        if (leaf && !map.has(leaf)) {
            map.set(leaf, stageIndex(topic.state));
        }
    }
    return map;
}

function mapAuthored(node: Node, stages: Map<string, number>): ConceptTreeNode {
    if (node.children.length === 0) {
        const stage = stages.get(node.title.trim().toLowerCase());
        return makeLeaf(node.id, node.title, stage ?? null);
    }
    return makeBranch(node.id, node.title, node.children.map((child) => mapAuthored(child, stages)));
}

// Working node for the fallback build: an ordered map keeps taxonomy display
// order (progress topics arrive in that order).
interface Building {
    id: string;
    title: string;
    children: Map<string, Building>;
    stage: number | null;
}

function finalizeBuilding(node: Building): ConceptTreeNode {
    if (node.children.size === 0) {
        return makeLeaf(node.id, node.title, node.stage);
    }
    return makeBranch(node.id, node.title, [...node.children.values()].map(finalizeBuilding));
}

// Rebuild a tree straight from the taxonomy `progress` paths, so an unauthored
// deck (the seed) still shows real structure. Each path segment is a branch and
// the last segment the leaf carrying its stage.
function fallbackTree(rootTitle: string, progress: SpeedrunProgress | null): ConceptTreeNode | null {
    const root: Building = { id: "root", title: rootTitle, children: new Map(), stage: null };
    for (const topic of progress?.topics ?? []) {
        const path = (topic.path ?? []).filter((seg) => seg.length > 0);
        const stage = stageIndex(topic.state);
        if (path.length === 0) {
            root.children.set(topic.topicId, { id: topic.topicId, title: topic.topicId, children: new Map(), stage });
            continue;
        }
        let cursor = root;
        path.forEach((seg, i) => {
            const existing = cursor.children.get(seg);
            const next: Building = existing
                ?? { id: `${cursor.id}/${seg}`, title: seg, children: new Map(), stage: null };
            if (!existing) {
                cursor.children.set(seg, next);
            }
            if (i === path.length - 1) {
                next.stage = stage;
            }
            cursor = next;
        });
    }
    const finalized = finalizeBuilding(root);
    if (finalized.children.length === 0) {
        return null;
    }
    // A single foundation is the natural root; drop the synthetic wrapper so the
    // seed deck reads "Biomolecules -> categories -> topics", not a stutter.
    return finalized.children.length === 1 ? finalized.children[0] : finalized;
}

// Build the concept tree: the deck's authored hierarchy for structure with the
// taxonomy stage overlaid on mapped leaves; fall back to the taxonomy paths
// when the deck has no authored children.
export function buildConceptTree(
    hierarchy: Hierarchy | null,
    progress: SpeedrunProgress | null,
): ConceptTreeNode | null {
    const root = hierarchy?.root ?? null;
    if (root && root.children.length > 0) {
        return mapAuthored(root, stagesByLeafLabel(progress));
    }
    return fallbackTree(root?.title || "All topics", progress);
}

// ---------------------------------------------------------------------------
// Per-subject breakdown (the stats modal)
// ---------------------------------------------------------------------------

export interface SubjectRow {
    id: string;
    name: string;
    // Fraction of the subject's leaves with at least one graded memory review.
    coverage: number;
    // Review-weighted mean FSRS retrievability over the subject's leaves.
    meanRetrievability: number;
    // Attempt-weighted application accuracy over the subject's leaves.
    applicationAccuracy: number;
    topicCount: number;
    memoryReviews: number;
    applicationAttempts: number;
    examWeight: number;
    hasMemoryData: boolean;
    hasApplicationData: boolean;
}

export interface SubjectBreakdown {
    subjects: SubjectRow[];
    // Any subject has memory or application evidence (else the modal shows a
    // "nothing practiced yet" note instead of empty meters).
    hasData: boolean;
}

function sum<T>(items: T[], pick: (item: T) => number): number {
    return items.reduce((total, item) => total + pick(item), 0);
}

// Roll the per-leaf breakdown up by content-category (the parent in `path`) into
// the subject meters the modal draws.
export function buildSubjectBreakdown(breakdown: SpeedrunScoreBreakdown | null): SubjectBreakdown {
    const groups = new Map<string, { name: string; leaves: SpeedrunScoreBreakdown_TopicStat[] }>();
    for (const topic of breakdown?.topics ?? []) {
        const path = topic.path ?? [];
        const parent = path.length >= 2 ? path.slice(0, -1) : path;
        const key = parent.length ? parent.join(" \u203a ") : "Other topics";
        const name = parent.length ? parent[parent.length - 1] : "Other topics";
        const group = groups.get(key);
        if (group) {
            group.leaves.push(topic);
        } else {
            groups.set(key, { name, leaves: [topic] });
        }
    }

    const subjects: SubjectRow[] = [...groups.entries()].map(([id, group]) => {
        const { leaves } = group;
        const memoryReviews = sum(leaves, (t) => t.memoryReviews);
        const applicationAttempts = sum(leaves, (t) => t.applicationAttempts);
        const covered = leaves.filter((t) => t.memoryReviews > 0).length;
        return {
            id,
            name: group.name,
            coverage: leaves.length > 0 ? covered / leaves.length : 0,
            meanRetrievability: memoryReviews > 0
                ? sum(leaves, (t) => t.meanRetrievability * t.memoryReviews) / memoryReviews
                : 0,
            applicationAccuracy: applicationAttempts > 0
                ? sum(leaves, (t) => t.applicationAccuracy * t.applicationAttempts) / applicationAttempts
                : 0,
            topicCount: leaves.length,
            memoryReviews,
            applicationAttempts,
            examWeight: sum(leaves, (t) => t.examWeight),
            hasMemoryData: memoryReviews > 0,
            hasApplicationData: leaves.some((t) => t.hasApplicationData),
        };
    });

    return {
        subjects,
        hasData: subjects.some((s) => s.hasMemoryData || s.hasApplicationData),
    };
}
