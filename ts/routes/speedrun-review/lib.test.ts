// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { expect, test } from "vitest";

import type { Concept, Node, Problem } from "../speedrun-hierarchy/lib";
import {
    findConcept,
    pathToNode,
    pickTwoProblems,
    rotateProblem,
    scaffoldSteps,
    shownProblem,
    STAGE_TOTAL,
    stageLabel,
    stageRank,
    upgradeLabels,
} from "./lib";

function problem(id: string, correctIndex: number): Problem {
    return {
        id,
        prompt: `prompt ${id}`,
        choices: [`${id}-a`, `${id}-b`, `${id}-c`, `${id}-d`],
        correctIndex,
    };
}

function concept(id: string, problems: Problem[]): Concept {
    return { id, title: `concept ${id}`, content: `content ${id}`, problems };
}

function node(id: string, title: string, children: Node[], concepts: Concept[] = []): Node {
    return { id, title, children, concepts };
}

test("shownProblem keeps the prompt and only the authored-correct choice", () => {
    expect(shownProblem(problem("p", 2))).toEqual({
        id: "p",
        prompt: "prompt p",
        answer: "p-c",
    });
    // An unmarked answer (-1) degrades to null rather than leaking a wrong choice.
    expect(shownProblem(problem("p", -1)).answer).toBeNull();
});

test("pickTwoProblems takes the first two, degrading for fewer", () => {
    const c = concept("c", [problem("p1", 0), problem("p2", 1), problem("p3", 2)]);
    const shown = pickTwoProblems(c);
    expect(shown.map((s) => s.id)).toEqual(["p1", "p2"]);
    expect(shown.map((s) => s.answer)).toEqual(["p1-a", "p2-b"]);

    expect(pickTwoProblems(concept("one", [problem("only", 3)]))).toHaveLength(1);
    expect(pickTwoProblems(concept("none", []))).toEqual([]);
});

test("rotateProblem cycles one problem per review and handles empties", () => {
    const problems = [problem("a", 0), problem("b", 1)];
    expect(rotateProblem(problems, 0)?.id).toBe("a");
    expect(rotateProblem(problems, 1)?.id).toBe("b");
    // Wraps back around after the last problem.
    expect(rotateProblem(problems, 2)?.id).toBe("a");
    expect(rotateProblem(problems, 3)?.id).toBe("b");
    // A negative rotation still lands in range.
    expect(rotateProblem(problems, -1)?.id).toBe("b");
    expect(rotateProblem([], 0)).toBeNull();
});

test("stageLabel maps engine states to display labels, hierarchy reads Applying", () => {
    expect(stageLabel("learning")).toBe("Learning");
    expect(stageLabel("practicing")).toBe("Practicing");
    expect(stageLabel("hierarchy")).toBe("Applying");
    expect(stageLabel("mastering")).toBe("Mastering");
    // An unknown state fails safe to the first rung.
    expect(stageLabel("bogus")).toBe("Learning");
});

test("stageRank orders the ladder and STAGE_TOTAL counts the rungs", () => {
    expect(STAGE_TOTAL).toBe(4);
    expect([
        stageRank("learning"),
        stageRank("practicing"),
        stageRank("hierarchy"),
        stageRank("mastering"),
    ]).toEqual([0, 1, 2, 3]);
    expect(stageRank("bogus")).toBe(0);
});

test("upgradeLabels renders the from/to of a stage change", () => {
    expect(upgradeLabels("learning", "practicing")).toEqual({
        from: "Learning",
        to: "Practicing",
    });
    expect(upgradeLabels("hierarchy", "mastering")).toEqual({
        from: "Applying",
        to: "Mastering",
    });
});

// A small authored tree: root -> (Foundation A -> Topic 1[cx], Topic 2) plus a
// concept sitting directly on the root, to exercise the shallow path.
function tree(): Node {
    const topic1 = node("t1", "Topic 1", [], [concept("cx", [problem("p", 0)])]);
    const topic2 = node("t2", "Topic 2", [], [concept("cy", [])]);
    const foundation = node("f1", "Foundation A", [topic1, topic2]);
    return node("root", "Deck", [foundation], [concept("cr", [])]);
}

test("findConcept locates a concept anywhere in the tree", () => {
    const root = tree();
    expect(findConcept(root, "cx")?.title).toBe("concept cx");
    expect(findConcept(root, "cr")?.title).toBe("concept cr");
    expect(findConcept(root, "missing")).toBeNull();
});

test("pathToNode returns the root-to-node ancestor chain", () => {
    const root = tree();
    expect(pathToNode(root, "t1")?.map((n) => n.id)).toEqual(["root", "f1", "t1"]);
    expect(pathToNode(root, "root")?.map((n) => n.id)).toEqual(["root"]);
    expect(pathToNode(root, "nope")).toBeNull();
});

test("scaffoldSteps walks root's children down to the concept's leaf", () => {
    const root = tree();
    const steps = scaffoldSteps(root, "cx");
    expect(steps).toHaveLength(2);
    // First pick: which foundation (only one here).
    expect(steps[0].options.map((o) => o.id)).toEqual(["f1"]);
    expect(steps[0].correctId).toBe("f1");
    // Second pick: which topic under the foundation.
    expect(steps[1].options.map((o) => o.id)).toEqual(["t1", "t2"]);
    expect(steps[1].correctId).toBe("t1");
});

test("scaffoldSteps is empty for a concept on the root or one not found", () => {
    const root = tree();
    expect(scaffoldSteps(root, "cr")).toEqual([]);
    expect(scaffoldSteps(root, "missing")).toEqual([]);
});
