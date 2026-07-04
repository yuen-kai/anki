// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { expect, test } from "vitest";

import {
    findNode,
    findParent,
    hasUnsavedChanges,
    isLeaf,
    isUnsaved,
    newConcept,
    newNode,
    newProblem,
    type SaveStatus,
    saveStatusLabel,
} from "./lib";

// A small three-level tree: deck -> group -> two leaves.
function fixture() {
    const leafA = newNode("Topic A");
    const leafB = newNode("Topic B");
    const group = newNode("Group");
    group.children = [leafA, leafB];
    const root = newNode("Deck");
    root.children = [group];
    return { root, group, leafA, leafB };
}

test("saveStatusLabel maps each status to its readout; clean is silent", () => {
    const labels: Record<SaveStatus, string> = {
        clean: "",
        dirty: "Unsaved",
        saving: "Saving",
        saved: "Saved",
        error: "Save failed",
    };
    for (const [status, label] of Object.entries(labels)) {
        expect(saveStatusLabel(status as SaveStatus)).toBe(label);
    }
});

test("hasUnsavedChanges guards leaving only for dirty or failed drafts", () => {
    // A save in flight already covers the current edits, and clean/saved drafts
    // have nothing to lose, so only these two prompt on the way out.
    expect(hasUnsavedChanges("dirty")).toBe(true);
    expect(hasUnsavedChanges("error")).toBe(true);
    expect(hasUnsavedChanges("clean")).toBe(false);
    expect(hasUnsavedChanges("saving")).toBe(false);
    expect(hasUnsavedChanges("saved")).toBe(false);
});

test("isUnsaved treats new and empty deck ids as 'create'", () => {
    expect(isUnsaved("")).toBe(true);
    expect(isUnsaved("new")).toBe(true);
    expect(isUnsaved("1700000000123")).toBe(false);
});

test("isLeaf is true only for a node with no children", () => {
    const { group, leafA } = fixture();
    expect(isLeaf(leafA)).toBe(true);
    expect(isLeaf(group)).toBe(false);
});

test("findNode walks the tree and returns null for a missing or null id", () => {
    const { root, group, leafB } = fixture();
    expect(findNode(root, root.id)).toBe(root);
    expect(findNode(root, group.id)).toBe(group);
    expect(findNode(root, leafB.id)).toBe(leafB);
    expect(findNode(root, "nope")).toBeNull();
    expect(findNode(root, null)).toBeNull();
});

test("findParent returns the immediate parent, never the node itself", () => {
    const { root, group, leafA } = fixture();
    // The group's parent is the deck root; the leaf's parent is the group.
    expect(findParent(root, group.id)).toBe(root);
    expect(findParent(root, leafA.id)).toBe(group);
    // The root has no parent, and a missing id resolves to null.
    expect(findParent(root, root.id)).toBeNull();
    expect(findParent(root, null)).toBeNull();
});

test("factory helpers mint distinct ids and empty content", () => {
    const a = newNode();
    const b = newNode();
    expect(a.id).not.toBe(b.id);
    expect(a.children).toEqual([]);
    expect(a.concepts).toEqual([]);

    const concept = newConcept();
    expect(concept.title).toBe("");
    expect(concept.problems).toEqual([]);

    const problem = newProblem();
    expect(problem.choices).toHaveLength(4);
    // No answer is marked until the author picks one.
    expect(problem.correctIndex).toBe(-1);
});
