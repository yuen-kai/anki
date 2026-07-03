// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { SpeedrunScoreBreakdown } from "@generated/anki/scheduler_pb";
import { expect, test } from "vitest";

import type { Hierarchy, Node } from "../speedrun-hierarchy/lib";
import type { StudyProgress } from "../speedrun-review/lib";
import {
    buildConceptTree,
    buildSubjectBreakdown,
    type ConceptTreeNode,
    ringDash,
    ringFraction,
    todayProgress,
} from "./lib";

function concept(id: string): Node["concepts"][number] {
    return { id, title: id, content: "", problems: [] };
}

function leaf(id: string, title: string, conceptIds: string[] = []): Node {
    return { id, title, children: [], concepts: conceptIds.map(concept) };
}

function child(node: ConceptTreeNode | undefined, title: string): ConceptTreeNode {
    const found = node?.children.find((c) => c.title === title);
    if (!found) {
        throw new Error(`missing child ${title}`);
    }
    return found;
}

test("buildConceptTree overlays authored per-concept stages on the hierarchy", () => {
    const hierarchy: Hierarchy = {
        deckId: "1",
        root: {
            id: "root",
            title: "Biochem",
            concepts: [],
            children: [
                {
                    id: "enz",
                    title: "Enzymes",
                    concepts: [],
                    children: [
                        leaf("l1", "Kinetics", ["k1"]),
                        leaf("l2", "Inhibition", ["i1"]),
                        leaf("l3", "Regulation", ["r1"]),
                    ],
                },
                { id: "pro", title: "Proteins", concepts: [], children: [leaf("l4", "Folding", ["f1"])] },
            ],
        },
    };
    const progress: StudyProgress = {
        k1: { state: "mastering", seen: true },
        i1: { state: "learning", seen: true },
        r1: { state: "hierarchy", seen: true },
        // f1 is absent, so Folding stays "not started".
    };

    const tree = buildConceptTree(hierarchy, progress)!;
    expect(tree.title).toBe("Biochem");
    expect(tree.isLeaf).toBe(false);
    expect(tree.leafCount).toBe(4);

    const enzymes = child(tree, "Enzymes");
    const kinetics = child(enzymes, "Kinetics");
    expect(kinetics.stage).toBe(3);
    expect(kinetics.stageLabel).toBe("Solo");
    expect(kinetics.fraction).toBeCloseTo(1);

    expect(child(enzymes, "Inhibition").stage).toBe(0);
    expect(child(enzymes, "Inhibition").fraction).toBeCloseTo(0.25);
    // The internal "hierarchy" state reads as "Guided".
    expect(child(enzymes, "Regulation").stageLabel).toBe("Guided");
    expect(child(enzymes, "Regulation").fraction).toBeCloseTo(0.75);

    const folding = child(child(tree, "Proteins"), "Folding");
    expect(folding.stage).toBeNull();
    expect(folding.stageLabel).toBe("Not started");
    expect(folding.fraction).toBe(0);

    // A branch aggregates the mean of its descendant leaf fractions.
    expect(enzymes.fraction).toBeCloseTo((1 + 0.25 + 0.75) / 3);
    expect(tree.fraction).toBeCloseTo((1 + 0.25 + 0.75 + 0) / 4);
});

test("a leaf's stage is the rounded mean over its concepts", () => {
    const hierarchy: Hierarchy = {
        deckId: "1",
        root: { id: "root", title: "Root", concepts: [], children: [leaf("l", "Mix", ["a", "b"])] },
    };
    // One mastered, one just started: the leaf reads as the mean (Guided).
    const progress: StudyProgress = {
        a: { state: "mastering", seen: true },
        b: { state: "learning", seen: true },
    };
    const mix = child(buildConceptTree(hierarchy, progress)!, "Mix");
    expect(mix.stage).toBe(2);
    expect(mix.stageLabel).toBe("Guided");
});

test("buildConceptTree is null when there is no authored structure", () => {
    expect(buildConceptTree(null, null)).toBeNull();
    const empty: Hierarchy = { deckId: "1", root: { id: "root", title: "x", children: [], concepts: [] } };
    expect(buildConceptTree(empty, null)).toBeNull();
});

test("buildSubjectBreakdown rolls per-leaf stats up by content-category", () => {
    const breakdown = new SpeedrunScoreBreakdown({
        topics: [
            {
                topicId: "k",
                path: ["Biomolecules", "Enzymes", "Kinetics"],
                meanRetrievability: 0.8,
                applicationAccuracy: 0.9,
                applicationAttempts: 10,
                memoryReviews: 20,
                examWeight: 0.18,
                hasApplicationData: true,
            },
            {
                topicId: "i",
                path: ["Biomolecules", "Enzymes", "Inhibition"],
                meanRetrievability: 0.6,
                applicationAccuracy: 0.5,
                applicationAttempts: 5,
                memoryReviews: 10,
                examWeight: 0.12,
                hasApplicationData: true,
            },
            // Untouched leaf: all zeros, no application data.
            {
                topicId: "r",
                path: ["Biomolecules", "Enzymes", "Regulation"],
                examWeight: 0.1,
                hasApplicationData: false,
            },
            {
                topicId: "s",
                path: ["Biomolecules", "Amino Acids", "Structure"],
                meanRetrievability: 0.7,
                memoryReviews: 5,
                examWeight: 0.15,
                hasApplicationData: false,
            },
        ],
    });

    const { subjects, hasData } = buildSubjectBreakdown(breakdown);
    expect(hasData).toBe(true);
    // Subjects keep taxonomy display order.
    expect(subjects.map((s) => s.name)).toEqual(["Enzymes", "Amino Acids"]);

    const enzymes = subjects.find((s) => s.name === "Enzymes")!;
    expect(enzymes.topicCount).toBe(3);
    expect(enzymes.memoryReviews).toBe(30);
    expect(enzymes.applicationAttempts).toBe(15);
    // Coverage: 2 of 3 leaves carry a graded review.
    expect(enzymes.coverage).toBeCloseTo(2 / 3);
    // Retrievability is review-weighted: (0.8*20 + 0.6*10) / 30.
    expect(enzymes.meanRetrievability).toBeCloseTo((0.8 * 20 + 0.6 * 10) / 30);
    // Accuracy is attempt-weighted: (0.9*10 + 0.5*5) / 15.
    expect(enzymes.applicationAccuracy).toBeCloseTo((0.9 * 10 + 0.5 * 5) / 15);
    expect(enzymes.examWeight).toBeCloseTo(0.4);
    expect(enzymes.hasApplicationData).toBe(true);

    const amino = subjects.find((s) => s.name === "Amino Acids")!;
    expect(amino.coverage).toBe(1);
    expect(amino.applicationAccuracy).toBe(0);
    expect(amino.hasApplicationData).toBe(false);
});

test("buildSubjectBreakdown reports no data for an empty breakdown", () => {
    expect(buildSubjectBreakdown(null)).toEqual({ subjects: [], hasData: false });
    const empty = buildSubjectBreakdown(new SpeedrunScoreBreakdown({ topics: [] }));
    expect(empty.hasData).toBe(false);
});

test("todayProgress: remaining is the headline, allDone when nothing is due", () => {
    const p = todayProgress({ deckName: "x", new: 3, learn: 1, review: 6, studiedToday: 10 });
    expect(p.remaining).toBe(10);
    expect(p.done).toBe(10);
    expect(p.total).toBe(20);
    expect(p.fraction).toBeCloseTo(0.5);
    expect(p.allDone).toBe(false);

    const done = todayProgress({ deckName: "x", new: 0, learn: 0, review: 0, studiedToday: 12 });
    expect(done.allDone).toBe(true);
    expect(done.fraction).toBe(1);

    // A fresh deck (nothing due, nothing done) reads as done, not divide-by-zero.
    expect(todayProgress(null).fraction).toBe(1);
});

test("ring geometry: fraction from the estimate, dash offset leaves only the fill", () => {
    // A points score fills by position on the 472-528 scale.
    expect(ringFraction({ estimate: 500, format: "points" } as never)).toBeCloseTo((500 - 472) / (528 - 472));
    // A ratio score fills by the estimate itself.
    expect(ringFraction({ estimate: 0.75, format: "ratio" } as never)).toBeCloseTo(0.75);

    const { circumference, offset } = ringDash(0.25, 50);
    expect(circumference).toBeCloseTo(2 * Math.PI * 50);
    expect(offset).toBeCloseTo(circumference * 0.75);
    // Out-of-range fractions are clamped.
    expect(ringDash(2, 50).offset).toBe(0);
});
