// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// Inline mock data for the study-screen demo. Nothing here touches the
// collection or the backend: the demo drives the real speedrun-review
// components with these fixed shapes so a developer can walk every screen,
// state, and animation without studying real cards. Shapes match the authored
// wire format in speedrun-hierarchy/lib.
import type { Concept, DeckSummary, Hierarchy, Node, Problem } from "../speedrun-hierarchy/lib";

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

// A short, display-only ledger for the authoring "Create deck" scene. The demo
// never lists or writes real decks, so these rows are static fixtures.
export const DEMO_DECKS: DeckSummary[] = [
    { deckId: "demo-mcat-physiology", name: "MCAT physiology", todo: 12 },
    { deckId: "demo-biochemistry", name: "Biochemistry", todo: 0 },
    { deckId: "demo-organic-chemistry", name: "Organic chemistry", todo: 5 },
];

// A private deep copy of the hierarchy for the authoring scenes. Those scenes
// reuse the real deck-editor components, which mutate their model in place, so
// cloning keeps the shared DEMO_HIERARCHY the study scenes read fully pristine.
export function cloneHierarchy(): Hierarchy {
    return JSON.parse(JSON.stringify(DEMO_HIERARCHY)) as Hierarchy;
}
