// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { goto } from "$app/navigation";
import {
    speedrunDeleteDeck,
    speedrunEnsureSeeded,
    speedrunGetHierarchy,
    speedrunListDecks,
    speedrunOpenDeck,
    speedrunSaveHierarchy,
} from "@generated/backend";
import { getContext, setContext } from "svelte";
import type { Writable } from "svelte/store";

// The Speedrun screens run in two hosts: the Qt desktop app (each screen is a
// separate webview state, so cross-screen navigation is a Qt `moveToState` RPC)
// and the AnkiDroid shell (one full-window webview running the whole SPA, so
// navigation is client-side SvelteKit routing). The host tags the mobile shell
// by setting `window.speedrunPlatform = "mobile"` before the app boots; on
// desktop it is unset. Navigation helpers branch on this so the same UI drives
// both without a per-platform build.
export function isMobileShell(): boolean {
    return (
        typeof window !== "undefined"
        && (window as unknown as { speedrunPlatform?: string }).speedrunPlatform === "mobile"
    );
}

// Frontend owns every id; the backend stores the blob verbatim, so these shapes
// are the single source of truth for the wire format too.
export interface Problem {
    id: string;
    prompt: string;
    choices: [string, string, string, string];
    // Index into `choices`; -1 while the author has not marked an answer yet.
    correctIndex: number;
    // Optional stem figure/crop, a filename in collection media, served
    // root-relative as `/<filename>`. Convention: `<problemId>-stem.png`.
    image?: string;
    // Optional per-choice figures, parallel to `choices` (length 4): a filename
    // or null. Convention: `<problemId>-c<0..3>.png`.
    choiceImages?: (string | null)[];
}

export interface Concept {
    id: string;
    title: string;
    content: string;
    problems: Problem[];
    // Optional source figure/crop, a filename in collection media, served
    // root-relative as `/<filename>`. Convention: `<conceptId>-src.png`.
    image?: string;
    // Optional short traceability snippet for where the concept came from.
    sourceText?: string;
}

export interface Node {
    id: string;
    title: string;
    children: Node[];
    // Only meaningful on leaves; a node with children reads as a branch.
    concepts: Concept[];
}

export interface Hierarchy {
    deckId: string;
    root: Node;
}

export interface DeckSummary {
    deckId: string;
    name: string;
    todo: number;
}

export interface SaveResult {
    deckId: string;
    name: string;
}

// The Speedrun JSON RPCs exchange a `{ json }` blob; these three helpers wrap
// the encode/decode and the "handle errors inline" option, shared by every
// Speedrun screen (hierarchy editor, study overview, review) so the plumbing
// lives once.
export const enc = (value: unknown): Uint8Array => new TextEncoder().encode(JSON.stringify(value));

export const dec = <T>(reply: { json: Uint8Array }): T => JSON.parse(new TextDecoder().decode(reply.json)) as T;

// The RPCs alert on error by default; every call site handles failures itself
// (inline status, retry), so we opt out of the global dialog.
export const quiet = { alertOnError: false } as const;

export async function listDecks(): Promise<DeckSummary[]> {
    return dec<DeckSummary[]>(await speedrunListDecks({ json: enc({}) }, quiet));
}

// Idempotently preload (or backfill from an updated bundle) the demo + MCAT
// decks through the shared engine. Desktop also seeds eagerly on collection open
// (qt main.py), so this is the mobile shell's trigger; returns the names of any
// decks it created or refreshed.
export async function ensureSeeded(): Promise<string[]> {
    const reply = dec<{ seeded?: string[]; updated?: string[] }>(
        await speedrunEnsureSeeded({ json: enc({}) }, quiet),
    );
    return [...(reply.seeded ?? []), ...(reply.updated ?? [])];
}

export async function getHierarchy(deckId: string): Promise<Hierarchy> {
    return dec<Hierarchy>(
        await speedrunGetHierarchy({ json: enc({ deckId }) }, quiet),
    );
}

export async function saveHierarchy(hierarchy: Hierarchy): Promise<SaveResult> {
    return dec<SaveResult>(await speedrunSaveHierarchy({ json: enc(hierarchy) }, quiet));
}

// Open a deck's study overview. Desktop moves the Qt window to the overview
// state; the mobile shell routes to the overview page in-place.
export async function openDeck(deckId: string): Promise<void> {
    if (isMobileShell()) {
        await goto(`/speedrun-study/${deckId}`);
        return;
    }
    await speedrunOpenDeck({ json: enc({ deckId }) }, quiet);
}

export async function deleteDeck(deckId: string): Promise<void> {
    await speedrunDeleteDeck({ json: enc({ deckId }) }, quiet);
}

export function newProblem(): Problem {
    return {
        id: crypto.randomUUID(),
        prompt: "",
        choices: ["", "", "", ""],
        correctIndex: -1,
    };
}

export function newConcept(): Concept {
    return { id: crypto.randomUUID(), title: "", content: "", problems: [] };
}

export function newNode(title = ""): Node {
    return { id: crypto.randomUUID(), title, children: [], concepts: [] };
}

export function isLeaf(node: Node): boolean {
    return node.children.length === 0;
}

export function findNode(root: Node, id: string | null): Node | null {
    if (!id) {
        return null;
    }
    if (root.id === id) {
        return root;
    }
    for (const child of root.children) {
        const found = findNode(child, id);
        if (found) {
            return found;
        }
    }
    return null;
}

// The immediate parent of a node, or null for the root / a missing id. Used for
// the concepts-panel breadcrumb ("Group 2 / Topic 4").
export function findParent(root: Node, id: string | null): Node | null {
    if (!id) {
        return null;
    }
    for (const child of root.children) {
        if (child.id === id) {
            return root;
        }
        const found = findParent(child, id);
        if (found) {
            return found;
        }
    }
    return null;
}

// A save creates the deck when it does not exist yet, so "new" and "" both mean
// "create". The backend echoes the real id back and we adopt it.
export function isUnsaved(deckId: string): boolean {
    return deckId === "" || deckId === "new";
}

// The save lifecycle for the builder header. A freshly loaded deck already lives
// on the backend, so it reads as "saved"; edits mark it "dirty"; an explicit
// save moves through "saving" to "saved", and a failed save lands on "error".
// "clean" is the silent start for a brand-new (unsaved) deck, which has nothing
// to report until it is touched.
export type SaveStatus = "clean" | "dirty" | "saving" | "saved" | "error";

// The mono readout shown beside the Save button. "clean" is intentionally silent.
export function saveStatusLabel(status: SaveStatus): string {
    switch (status) {
        case "saving":
            return "Saving";
        case "saved":
            return "Saved";
        case "dirty":
            return "Unsaved";
        case "error":
            return "Save failed";
        default:
            return "";
    }
}

// Whether the draft holds edits that are not on the backend yet, so leaving the
// screen should warn first. A save in flight already covers the current edits;
// only an unsaved ("dirty") or failed ("error") draft needs the guard.
export function hasUnsavedChanges(status: SaveStatus): boolean {
    return status === "dirty" || status === "error";
}

// A pulse names the ancestor chain (edited node first, root last) plus a
// monotonic sequence so a repeat edit restarts the animation.
export interface PulseState {
    ids: string[];
    seq: number;
}

// Shared editor plumbing handed to the recursive tree and the concepts panel,
// so a change anywhere can mark the draft dirty, drive selection, and the pulse
// without threading callbacks through every level.
export interface TreeContext {
    change(): void;
    pulse(chain: string[]): void;
    select(id: string | null): void;
    selectedId: Writable<string | null>;
    pulseState: Writable<PulseState | null>;
}

const TREE_KEY = Symbol("speedrun-tree");

export function setTreeContext(context: TreeContext): void {
    setContext(TREE_KEY, context);
}

export function getTreeContext(): TreeContext {
    return getContext<TreeContext>(TREE_KEY);
}
