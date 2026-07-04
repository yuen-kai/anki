// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// Inline mock data for the study-screen demo. Nothing here touches the
// collection or the backend: the demo drives the real speedrun-review
// components with these fixed shapes so a developer can walk every screen,
// state, and animation without studying real cards. Shapes match the authored
// wire format in speedrun-hierarchy/lib.
import { deferredEnvelope, type ScoreEnvelope } from "../speedrun-dashboard/lib";
import type { DeckRow } from "../speedrun-decks/lib";
import type { Concept, Hierarchy, Node, Problem } from "../speedrun-hierarchy/lib";
import type { StudyProgress } from "../speedrun-review/lib";
import {
    buildConceptTree,
    type ConceptTreeNode,
    type StudySummary,
    type SubjectBreakdown,
} from "../speedrun-study/lib";

// Stable ids so the demo page can name specific nodes/concepts (the intro
// target, the scaffolded concept) without walking the tree by title.
export const DEMO_IDS = {
    root: "demo-root",
    cardiovascular: "demo-cv",
    electrophysiology: "demo-cv-ep",
    hemodynamics: "demo-cv-hemo",
    renal: "demo-renal",
    glomerulus: "demo-renal-glom",
    tubule: "demo-renal-tub",
    actionPotential: "demo-c-ap",
    refractory: "demo-c-refractory",
    cardiacOutput: "demo-c-output",
    filtrationBarrier: "demo-c-barrier",
    netPressure: "demo-c-netpressure",
    sodium: "demo-c-sodium",
} as const;

function problem(
    id: string,
    prompt: string,
    choices: [string, string, string, string],
    correctIndex: number,
): Problem {
    return { id, prompt, choices, correctIndex };
}

const actionPotential: Concept = {
    id: DEMO_IDS.actionPotential,
    title: "Cardiac action potential",
    content: "Ventricular myocytes fire a five-phase action potential. Phase 0 is a fast "
        + "sodium influx; phase 2 is a calcium plateau that sustains contraction and "
        + "keeps the cell refractory far longer than skeletal muscle does.",
    problems: [
        problem(
            "demo-p-ap-1",
            "What carries the rapid phase 0 upstroke in a ventricular myocyte?",
            ["Potassium efflux", "Chloride influx", "Sodium influx", "Calcium influx"],
            2,
        ),
        problem(
            "demo-p-ap-2",
            "The phase 2 plateau is held up mainly by:",
            [
                "Calcium influx balancing potassium efflux",
                "A second sodium spike",
                "Every ion channel closing at once",
                "A chloride influx",
            ],
            0,
        ),
        problem(
            "demo-p-ap-3",
            "Why can cardiac muscle not be tetanized?",
            [
                "It has no troponin",
                "It lacks a sarcoplasmic reticulum",
                "It contracts voluntarily",
                "Its refractory period outlasts the twitch",
            ],
            3,
        ),
    ],
};

const refractory: Concept = {
    id: DEMO_IDS.refractory,
    title: "Refractory periods",
    content: "During the absolute refractory period no stimulus can start a second action "
        + "potential, because the fast sodium channels are inactivated. This keeps each "
        + "heartbeat discrete and blocks summation.",
    problems: [
        problem(
            "demo-p-ref-1",
            "During the absolute refractory period, a second action potential is:",
            [
                "Triggered by a strong enough stimulus",
                "Impossible regardless of stimulus strength",
                "Always spontaneous",
                "Faster than the first",
            ],
            1,
        ),
        problem(
            "demo-p-ref-2",
            "The refractory period exists because:",
            [
                "Fast sodium channels are inactivated",
                "Calcium is fully depleted",
                "ATP has run out",
                "The membrane stays hyperpolarized forever",
            ],
            0,
        ),
        problem(
            "demo-p-ref-3",
            "A practical consequence of the long refractory period is that:",
            [
                "Action potentials summate",
                "Conduction reverses direction",
                "The heart cannot sustain a tetanic contraction",
                "The heart speeds up under load",
            ],
            2,
        ),
    ],
};

const cardiacOutput: Concept = {
    id: DEMO_IDS.cardiacOutput,
    title: "Cardiac output",
    content: "Cardiac output is heart rate times stroke volume. Stroke volume rises with "
        + "preload (Frank-Starling) and contractility, and falls as afterload climbs.",
    problems: [
        problem(
            "demo-p-out-1",
            "Cardiac output equals:",
            [
                "Stroke volume divided by heart rate",
                "Heart rate times stroke volume",
                "Preload times afterload",
                "Mean pressure times resistance",
            ],
            1,
        ),
        problem(
            "demo-p-out-2",
            "By the Frank-Starling relationship, greater preload gives:",
            ["A larger stroke volume", "A smaller stroke volume", "No change", "A slower heart rate"],
            0,
        ),
        problem(
            "demo-p-out-3",
            "Raising afterload while everything else is fixed tends to give:",
            [
                "A higher stroke volume",
                "A faster heart rate",
                "No change in output",
                "A lower stroke volume",
            ],
            3,
        ),
    ],
};

const filtrationBarrier: Concept = {
    id: DEMO_IDS.filtrationBarrier,
    title: "Filtration barrier",
    content: "The glomerular filtration barrier has three layers: the fenestrated "
        + "endothelium, the basement membrane, and the podocyte slit diaphragms. Together "
        + "they pass water and small solutes but hold back plasma proteins.",
    problems: [
        problem(
            "demo-p-bar-1",
            "Which layer chiefly excludes plasma proteins?",
            ["The basement membrane", "The bladder wall", "The proximal tubule", "The collecting duct"],
            0,
        ),
        problem(
            "demo-p-bar-2",
            "Normal glomerular filtrate is essentially free of:",
            ["Sodium", "Glucose", "Large plasma proteins", "Water"],
            2,
        ),
        problem(
            "demo-p-bar-3",
            "Podocyte foot processes form the:",
            [
                "Wall of the afferent arteriole",
                "Slit diaphragms of the barrier",
                "Loop of Henle",
                "Lining of the bladder",
            ],
            1,
        ),
    ],
};

const netPressure: Concept = {
    id: DEMO_IDS.netPressure,
    title: "Net filtration pressure",
    content: "Net filtration pressure is glomerular capillary hydrostatic pressure minus "
        + "Bowman's space pressure and the blood colloid osmotic pressure. It sets the "
        + "glomerular filtration rate.",
    problems: [
        problem(
            "demo-p-net-1",
            "Net filtration pressure is driven mainly by:",
            [
                "Glomerular capillary hydrostatic pressure",
                "Bowman's space hydrostatic pressure",
                "Blood colloid osmotic pressure",
                "Tubular sodium",
            ],
            0,
        ),
        problem(
            "demo-p-net-2",
            "Constricting the efferent arteriole tends to:",
            [
                "Raise glomerular pressure and GFR",
                "Lower glomerular pressure and GFR",
                "Leave GFR unchanged",
                "Stop filtration entirely",
            ],
            0,
        ),
        problem(
            "demo-p-net-3",
            "A rise in blood colloid osmotic pressure will:",
            [
                "Raise net filtration pressure",
                "Rupture the barrier",
                "Lower net filtration pressure",
                "Have no effect",
            ],
            2,
        ),
    ],
};

const sodium: Concept = {
    id: DEMO_IDS.sodium,
    title: "Sodium reabsorption",
    content: "Most filtered sodium is reclaimed in the proximal tubule, coupled to glucose "
        + "and amino acids, then fine-tuned in the distal nephron under aldosterone. "
        + "Sodium handling sets the body's water and blood-pressure balance.",
    problems: [
        problem(
            "demo-p-na-1",
            "The bulk of filtered sodium is reabsorbed in the:",
            ["The bladder", "The glomerulus", "The collecting duct", "The proximal tubule"],
            3,
        ),
        problem(
            "demo-p-na-2",
            "Aldosterone increases sodium reabsorption in the:",
            [
                "Proximal brush border",
                "Distal nephron",
                "Afferent arteriole",
                "Bowman's capsule",
            ],
            1,
        ),
        problem(
            "demo-p-na-3",
            "Proximal sodium reabsorption is coupled to the uptake of:",
            ["Glucose and amino acids", "Plasma proteins", "Red blood cells", "Urea only"],
            0,
        ),
    ],
};

function leaf(id: string, title: string, concepts: Concept[]): Node {
    return { id, title, children: [], concepts };
}

function branch(id: string, title: string, children: Node[]): Node {
    return { id, title, children, concepts: [] };
}

// Two top-level topics (Cardiovascular, Renal), each 3 concepts spread over two
// leaves. The two-level depth gives the Applying scaffold real steps to pick,
// and each branch has two children so every scaffold step offers a real choice.
export const DEMO_HIERARCHY: Hierarchy = {
    deckId: "demo",
    root: branch(DEMO_IDS.root, "MCAT physiology", [
        branch(DEMO_IDS.cardiovascular, "Cardiovascular", [
            leaf(DEMO_IDS.electrophysiology, "Cardiac electrophysiology", [
                actionPotential,
                refractory,
            ]),
            leaf(DEMO_IDS.hemodynamics, "Hemodynamics", [cardiacOutput]),
        ]),
        branch(DEMO_IDS.renal, "Renal", [
            leaf(DEMO_IDS.glomerulus, "Glomerulus", [filtrationBarrier, netPressure]),
            leaf(DEMO_IDS.tubule, "Tubule", [sodium]),
        ]),
    ]),
};

// The intro highlights the path down to this leaf (root -> Cardiovascular ->
// Cardiac electrophysiology).
export const DEMO_TOPIC_NODE_ID = DEMO_IDS.electrophysiology;

// The learning block the demo teaches: the Cardiovascular topic's three
// concepts, so the "N / total concepts" counter climbs 1..3.
export const DEMO_LEARNING_CONCEPTS: Concept[] = [actionPotential, refractory, cardiacOutput];

// The concept the Applying/Mastering scenes drill. It sits two levels deep, so
// scaffoldSteps() returns two real picks.
export const DEMO_CONCEPT_ID = DEMO_IDS.actionPotential;

// Display-only rows for the Decks home scene. The demo never lists or writes
// real decks, so these carry fixed derived counts; the first row mirrors the
// authored DEMO_HIERARCHY (4 topics · 6 concepts).
export const DEMO_DECKS: DeckRow[] = [
    {
        deckId: "demo-mcat-physiology",
        name: "MCAT physiology",
        todo: 9,
        metrics: { topics: 4, concepts: 6, completion: 0.62 },
    },
    {
        deckId: "demo-biochemistry",
        name: "Biochemistry",
        todo: 0,
        metrics: { topics: 8, concepts: 12, completion: 0.3 },
    },
    {
        deckId: "demo-organic-chemistry",
        name: "Organic chemistry",
        todo: 5,
        metrics: { topics: 5, concepts: 9, completion: 0 },
    },
];

// A private deep copy of the hierarchy for the authoring scenes. Those scenes
// reuse the real deck-editor components, which mutate their model in place, so
// cloning keeps the shared DEMO_HIERARCHY the study scenes read fully pristine.
export function cloneHierarchy(): Hierarchy {
    return JSON.parse(JSON.stringify(DEMO_HIERARCHY)) as Hierarchy;
}

// ---------------------------------------------------------------------------
// Study overview (canvas 2b delivered / 2i not-started)
// ---------------------------------------------------------------------------

// Per-concept study progress powering the delivered overview's tree + rollups:
// a spread of stages so the leaves show the mastery ladder's full colour range.
const DEMO_PROGRESS: StudyProgress = {
    [DEMO_IDS.actionPotential]: { state: "hierarchy", seen: true },
    [DEMO_IDS.refractory]: { state: "practicing", seen: true },
    [DEMO_IDS.cardiacOutput]: { state: "practicing", seen: true },
    [DEMO_IDS.filtrationBarrier]: { state: "mastering", seen: true },
    [DEMO_IDS.netPressure]: { state: "hierarchy", seen: true },
    [DEMO_IDS.sodium]: { state: "practicing", seen: true },
};

// The props the real StudyOverview reads (minus its handlers), so the demo can
// spread one fixture straight onto the component.
export interface DemoStudyData {
    summary: StudySummary;
    memory: ScoreEnvelope | null;
    performance: ScoreEnvelope | null;
    readiness: ScoreEnvelope | null;
    tree: ConceptTreeNode | null;
    subjects: SubjectBreakdown;
    filtered: boolean;
}

// Delivered overview: Memory has graded evidence (a likely range + coverage +
// driver lines); Performance and Readiness still await scored application work,
// mirroring a deck studied to Practice but not yet drilled — the same mix the
// real seeded-deck screenshot shows.
export const DEMO_STUDY: DemoStudyData = {
    summary: { deckName: "MCAT physiology", new: 9, learn: 0, review: 0, studiedToday: 6 },
    memory: {
        estimate: 0.87,
        rangeLow: 0.84,
        rangeHigh: 0.91,
        coveragePct: 1,
        confidence: "",
        updatedAtSecs: 0,
        reasons: ["Strong: Cardiac electrophysiology", "Weak: Hemodynamics"],
        abstained: false,
        abstainReason: "",
        gradedReviews: 24,
        format: "ratio",
    },
    performance: deferredEnvelope("needs 5 application attempts, have 0; coverage 0% below the 25% minimum"),
    readiness: deferredEnvelope(
        "needs the Performance score first: needs 5 application attempts, have 0; coverage 0% below the 25% minimum",
    ),
    tree: buildConceptTree(DEMO_HIERARCHY, DEMO_PROGRESS),
    subjects: {
        subjects: [
            {
                id: "cardiovascular",
                name: "Cardiovascular",
                coverage: 1,
                meanRetrievability: 0.9,
                applicationAccuracy: 0,
                topicCount: 2,
                memoryReviews: 16,
                applicationAttempts: 0,
                examWeight: 0.45,
                hasMemoryData: true,
                hasApplicationData: false,
            },
            {
                id: "renal",
                name: "Renal",
                coverage: 0.5,
                meanRetrievability: 0.82,
                applicationAccuracy: 0,
                topicCount: 2,
                memoryReviews: 8,
                applicationAttempts: 0,
                examWeight: 0.55,
                hasMemoryData: true,
                hasApplicationData: false,
            },
        ],
        hasData: true,
    },
    filtered: false,
};

// Not-started edge: nothing due, every score locked at "—" with its missing-data
// reason, and a tree with no graded progress yet.
export const DEMO_STUDY_NOTSTARTED: DemoStudyData = {
    summary: { deckName: "MCAT physiology", new: 0, learn: 0, review: 0, studiedToday: 0 },
    memory: deferredEnvelope("needs 5 graded reviews, have 0; coverage 0% below the 25% minimum"),
    performance: deferredEnvelope("needs 5 application attempts, have 0; coverage 0% below the 25% minimum"),
    readiness: deferredEnvelope(
        "needs the Performance score first: needs 5 application attempts, have 0; coverage 0% below the 25% minimum",
    ),
    tree: buildConceptTree(DEMO_HIERARCHY, {}),
    subjects: {
        subjects: [
            {
                id: "cardiovascular",
                name: "Cardiovascular",
                coverage: 0,
                meanRetrievability: 0,
                applicationAccuracy: 0,
                topicCount: 2,
                memoryReviews: 0,
                applicationAttempts: 0,
                examWeight: 0.45,
                hasMemoryData: false,
                hasApplicationData: false,
            },
            {
                id: "renal",
                name: "Renal",
                coverage: 0,
                meanRetrievability: 0,
                applicationAccuracy: 0,
                topicCount: 2,
                memoryReviews: 0,
                applicationAttempts: 0,
                examWeight: 0.55,
                hasMemoryData: false,
                hasApplicationData: false,
            },
        ],
        hasData: false,
    },
    filtered: false,
};
