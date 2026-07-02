// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import {
    speedrunDeleteDeck,
    speedrunGetHierarchy,
    speedrunListDecks,
    speedrunOpenDeck,
    speedrunSaveHierarchy,
} from "@generated/backend";
import { getContext, setContext } from "svelte";
import type { Writable } from "svelte/store";

// Frontend owns every id; the backend stores the blob verbatim, so these shapes
// are the single source of truth for the wire format too.
export interface Problem {
    id: string;
    prompt: string;
    choices: [string, string, string, string];
    // Index into `choices`; -1 while the author has not marked an answer yet.
    correctIndex: number;
}

export interface Concept {
    id: string;
    title: string;
    content: string;
    problems: Problem[];
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

const enc = (value: unknown): Uint8Array => new TextEncoder().encode(JSON.stringify(value));

const dec = <T>(reply: { json: Uint8Array }): T => JSON.parse(new TextDecoder().decode(reply.json)) as T;

// The RPCs alert on error by default; every call site here handles failures
// itself (inline status, retry), so we opt out of the global dialog.
const quiet = { alertOnError: false } as const;

export async function listDecks(): Promise<DeckSummary[]> {
    return dec<DeckSummary[]>(await speedrunListDecks({ json: enc({}) }, quiet));
}

export async function getHierarchy(deckId: string): Promise<Hierarchy> {
    return dec<Hierarchy>(
        await speedrunGetHierarchy({ json: enc({ deckId }) }, quiet),
    );
}

export async function saveHierarchy(hierarchy: Hierarchy): Promise<SaveResult> {
    return dec<SaveResult>(await speedrunSaveHierarchy({ json: enc(hierarchy) }, quiet));
}

export async function openDeck(deckId: string): Promise<void> {
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

// A save creates the deck when it does not exist yet, so "new" and "" both mean
// "create". The backend echoes the real id back and we adopt it.
export function isUnsaved(deckId: string): boolean {
    return deckId === "" || deckId === "new";
}

export const AUTOSAVE_MS = 500;

export interface Autosave {
    schedule(hierarchy: Hierarchy): void;
    // Save any pending edit right now (e.g. on unmount) instead of waiting out
    // the debounce.
    flush(): void;
    cancel(): void;
}

// Debounced, self-coalescing autosave: edits queue behind an in-flight save
// instead of racing it, so the last write always wins and we never overlap
// requests.
export function createAutosave(
    onResult: (result: SaveResult) => void,
    onError: (error: unknown) => void,
    delayMs = AUTOSAVE_MS,
): Autosave {
    let timer: ReturnType<typeof setTimeout> | undefined;
    let saving = false;
    let dirty = false;
    let latest: Hierarchy | null = null;

    async function run(): Promise<void> {
        if (!latest) {
            return;
        }
        saving = true;
        dirty = false;
        try {
            onResult(await saveHierarchy(latest));
        } catch (error) {
            onError(error);
        } finally {
            saving = false;
            if (dirty) {
                run();
            }
        }
    }

    return {
        schedule(hierarchy: Hierarchy): void {
            latest = hierarchy;
            if (saving) {
                dirty = true;
                return;
            }
            if (timer) {
                clearTimeout(timer);
            }
            timer = setTimeout(() => {
                timer = undefined;
                run();
            }, delayMs);
        },
        flush(): void {
            if (timer) {
                clearTimeout(timer);
                timer = undefined;
            }
            if (saving) {
                dirty = true;
                return;
            }
            run();
        },
        cancel(): void {
            if (timer) {
                clearTimeout(timer);
                timer = undefined;
            }
        },
    };
}

// A pulse names the ancestor chain (edited node first, root last) plus a
// monotonic sequence so a repeat edit restarts the animation.
export interface PulseState {
    ids: string[];
    seq: number;
}

// Shared editor plumbing handed to the recursive tree and the concepts panel,
// so a change anywhere can trigger autosave, selection, and the pulse without
// threading callbacks through every level.
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
