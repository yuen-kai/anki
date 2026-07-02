// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import {
    speedrunAnswerCard,
    speedrunNextCard,
    speedrunRecordLearned,
    speedrunShowDecks,
    speedrunStudyState,
} from "@generated/backend";

import { MASTERY_STAGES } from "../speedrun-dashboard/lib";
import {
    type Concept,
    getHierarchy,
    type Hierarchy,
    type Node,
    openDeck,
    type Problem,
} from "../speedrun-hierarchy/lib";

// Re-export the authored shapes and the two exits (deck overview / decks home)
// so the screen imports everything study-related from one place; the study RPCs
// never send card/concept content, we look it up here.
export { getHierarchy, openDeck };
export type { Concept, Hierarchy, Node, Problem };

// The internal mastery states, low to high. `hierarchy` is the engine's name;
// it DISPLAYS as "Applying" (see stageLabel).
export type StudyStateName = "learning" | "practicing" | "hierarchy" | "mastering";

export interface ConceptProgress {
    state: StudyStateName;
    seen: boolean;
}

// deckId -> the full per-concept map; an absent concept means learning/unseen.
export type StudyProgress = Record<string, ConceptProgress>;

export interface StudyStateResponse {
    progress: StudyProgress;
}

// A whole topic (authored leaf) taught as one block before any grading.
export interface LearningBlock {
    kind: "learning_block";
    topicNodeId: string;
    conceptIds: string[];
    learnedCount: number;
    totalCount: number;
}

// One FSRS-due concept card; `state` selects the interaction to render.
export interface ReviewCard {
    kind: "review";
    cardId: string;
    conceptId: string;
    state: StudyStateName;
}

export interface DoneCard {
    kind: "done";
}

export type NextCard = LearningBlock | ReviewCard | DoneCard;

// What a grade returns: the concept's new state plus the from/to of any stage
// change so the screen can play the upgrade animation.
export interface AnswerResult {
    state: StudyStateName;
    upgraded: boolean;
    from: StudyStateName;
    to: StudyStateName;
}

// What marking a concept learned returns: `conceptIds` are the ones flipped
// learning -> practicing this call (empty until the whole topic is seen).
export interface LearnedResult {
    upgraded: boolean;
    from: StudyStateName;
    to: StudyStateName;
    conceptIds: string[];
}

// 1..4 = Again/Hard/Good/Easy, fed straight to FSRS by the backend.
export type Rating = 1 | 2 | 3 | 4;

export interface RatingChoice {
    rating: Rating;
    label: string;
}

// The four grades, in the order base Anki lays them out (worst to best).
export const RATINGS: RatingChoice[] = [
    { rating: 1, label: "Again" },
    { rating: 2, label: "Hard" },
    { rating: 3, label: "Good" },
    { rating: 4, label: "Easy" },
];

const enc = (value: unknown): Uint8Array => new TextEncoder().encode(JSON.stringify(value));

const dec = <T>(reply: { json: Uint8Array }): T => JSON.parse(new TextDecoder().decode(reply.json)) as T;

// The screen shows its own inline status/finished states, so it opts out of the
// backend's global error dialog (same choice as the authoring RPCs).
const quiet = { alertOnError: false } as const;

export async function studyState(deckId: string): Promise<StudyStateResponse> {
    return dec<StudyStateResponse>(await speedrunStudyState({ json: enc({ deckId }) }, quiet));
}

export async function nextCard(deckId: string): Promise<NextCard> {
    return dec<NextCard>(await speedrunNextCard({ json: enc({ deckId }) }, quiet));
}

export async function answerCard(
    deckId: string,
    cardId: string,
    conceptId: string,
    rating: Rating,
): Promise<AnswerResult> {
    return dec<AnswerResult>(
        await speedrunAnswerCard({ json: enc({ deckId, cardId, conceptId, rating }) }, quiet),
    );
}

export async function recordLearned(deckId: string, conceptIds: string[]): Promise<LearnedResult> {
    return dec<LearnedResult>(
        await speedrunRecordLearned({ json: enc({ deckId, conceptIds }) }, quiet),
    );
}

// Leave the study screen for the decks home; the backend moves the window.
export async function showDecks(): Promise<void> {
    await speedrunShowDecks({ json: enc({}) }, quiet);
}

// --- Pure helpers (unit-tested in lib.test.ts) -----------------------------

// A problem stripped to what the learning stage shows: the prompt and ONLY the
// authored-correct choice. `answer` is null when the author never marked one.
export interface ShownProblem {
    id: string;
    prompt: string;
    answer: string | null;
}

export function shownProblem(problem: Problem): ShownProblem {
    const i = problem.correctIndex;
    const answer = i >= 0 && i < problem.choices.length ? problem.choices[i] : null;
    return { id: problem.id, prompt: problem.prompt, answer };
}

// The learning stage teaches a concept from its first two problems (contrasting
// cases). Fewer than two degrades gracefully to whatever exists.
export function pickTwoProblems(concept: Concept): ShownProblem[] {
    return concept.problems.slice(0, 2).map(shownProblem);
}

// Applying/Mastering rotate through the concept's problems one per review, next
// unseen then cycling; `rotation` is the count of reviews already shown, so the
// first review is index 0. Null when the concept authored no problems.
export function rotateProblem(problems: Problem[], rotation: number): Problem | null {
    if (problems.length === 0) {
        return null;
    }
    const index = ((rotation % problems.length) + problems.length) % problems.length;
    return problems[index];
}

const STAGE_LABEL = new Map<string, string>(
    MASTERY_STAGES.map((stage) => [stage.state, stage.label] as const),
);

const STAGE_BLURB = new Map<string, string>(
    MASTERY_STAGES.map((stage) => [stage.state, stage.blurb] as const),
);

// learning->Learning, practicing->Practicing, hierarchy->Applying,
// mastering->Mastering. An unknown state falls back to the first rung.
export function stageLabel(state: string): string {
    return STAGE_LABEL.get(state) ?? MASTERY_STAGES[0].label;
}

// The number of rungs on the mastery ladder (learning..mastering).
export const STAGE_TOTAL = MASTERY_STAGES.length;

// The 0-based rung of a state, for drawing the mastery meter. Unknown = 0.
export function stageRank(state: string): number {
    const i = MASTERY_STAGES.findIndex((stage) => stage.state === state);
    return i < 0 ? 0 : i;
}

export function stageBlurb(state: string): string {
    return STAGE_BLURB.get(state) ?? MASTERY_STAGES[0].blurb;
}

export interface UpgradeLabels {
    from: string;
    to: string;
}

// The display labels for an upgrade animation, e.g. Learning -> Practicing.
export function upgradeLabels(from: string, to: string): UpgradeLabels {
    return { from: stageLabel(from), to: stageLabel(to) };
}

function walkNodes(node: Node): Node[] {
    return [node, ...node.children.flatMap(walkNodes)];
}

// The concept object (title, content, problems) looked up by id, or null.
export function findConcept(root: Node, conceptId: string): Concept | null {
    for (const node of walkNodes(root)) {
        const found = node.concepts.find((concept) => concept.id === conceptId);
        if (found) {
            return found;
        }
    }
    return null;
}

// The ancestor chain [root, ..., leaf] down to the node matching `predicate`,
// or null. Root is always first so callers can drop it when they want the
// visible path.
function pathBy(node: Node, predicate: (node: Node) => boolean): Node[] | null {
    if (predicate(node)) {
        return [node];
    }
    for (const child of node.children) {
        const sub = pathBy(child, predicate);
        if (sub) {
            return [node, sub].flat();
        }
    }
    return null;
}

// [root, ..., targetNode] for a node id; used to animate the New Topic path.
export function pathToNode(root: Node, nodeId: string): Node[] | null {
    return pathBy(root, (node) => node.id === nodeId);
}

// [root, ..., leaf] for the leaf that holds the concept; used by the scaffold.
export function pathToConcept(root: Node, conceptId: string): Node[] | null {
    return pathBy(root, (node) => node.concepts.some((concept) => concept.id === conceptId));
}

export interface ScaffoldOption {
    id: string;
    title: string;
}

export interface ScaffoldStep {
    // The siblings at this level; the learner picks the one on the path.
    options: ScaffoldOption[];
    correctId: string;
}

// The Applying-stage scaffold: pick down the authored tree from root's children
// to the concept's leaf, options = siblings at each level. Empty when the
// concept sits on the root (nothing to place) or is not found.
export function scaffoldSteps(root: Node, conceptId: string): ScaffoldStep[] {
    const path = pathToConcept(root, conceptId);
    if (!path || path.length < 2) {
        return [];
    }
    const steps: ScaffoldStep[] = [];
    for (let i = 1; i < path.length; i++) {
        steps.push({
            options: path[i - 1].children.map((child) => ({ id: child.id, title: child.title })),
            correctId: path[i].id,
        });
    }
    return steps;
}
