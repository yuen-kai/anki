// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { SpeedrunProgress } from "@generated/anki/scheduler_pb";
import { expect, test } from "vitest";

import { buildTopicsView, stageIndex } from "./lib";

test("stageIndex maps the four states in ladder order", () => {
    expect(stageIndex("learning")).toBe(0);
    expect(stageIndex("practicing")).toBe(1);
    expect(stageIndex("hierarchy")).toBe(2);
    expect(stageIndex("mastering")).toBe(3);
    // Unknown/absent is the fail-safe first rung (matches the engine default),
    // never a negative index.
    expect(stageIndex("")).toBe(0);
    expect(stageIndex("bogus")).toBe(0);
});

test("buildTopicsView groups by hierarchy and counts the distribution", () => {
    const view = buildTopicsView(
        new SpeedrunProgress({
            topics: [
                { topicId: "a", state: "mastering", path: ["Biomolecules", "Enzymes", "Inhibition"] },
                { topicId: "b", state: "learning", path: ["Biomolecules", "Enzymes", "Kinetics"] },
                { topicId: "c", state: "hierarchy", path: ["Biomolecules", "Amino acids", "Structure"] },
            ],
        }),
    );

    expect(view.total).toBe(3);
    // Index-aligned with the ladder [learning, practicing, hierarchy, mastering].
    expect(view.distribution).toEqual([1, 0, 1, 1]);
    // Two category groups, sorted by heading.
    expect(view.groups.map((g) => g.heading)).toEqual([
        "Biomolecules › Amino acids",
        "Biomolecules › Enzymes",
    ]);
    // Topics within a group are sorted by label and carry their stage label.
    const enzymes = view.groups.find((g) => g.heading === "Biomolecules › Enzymes")!;
    expect(enzymes.topics.map((t) => t.label)).toEqual(["Inhibition", "Kinetics"]);
    expect(enzymes.topics.map((t) => t.stageLabel)).toEqual(["Mastering", "Learning"]);
});

test("buildTopicsView is empty for null and falls back when a topic has no path", () => {
    expect(buildTopicsView(null).total).toBe(0);
    expect(buildTopicsView(null).distribution).toEqual([0, 0, 0, 0]);

    const view = buildTopicsView(
        new SpeedrunProgress({
            topics: [{ topicId: "mcat::x::y", state: "practicing", path: [] }],
        }),
    );
    expect(view.total).toBe(1);
    expect(view.groups[0].heading).toBe("Other topics");
    expect(view.groups[0].topics[0].label).toBe("mcat::x::y");
    expect(view.groups[0].topics[0].stageLabel).toBe("Practicing");
});
