<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Guided: one card, disclosed downward. Step ① locate the concept in the
hierarchy (Group → Topic chips); then, revealed below via the seam, Step ② pick
the answer and Step ③ its checked result (the Mcq body); then Step ④ grading at
the bottom. Solo is the same question with no locate step (ProblemCard), so the
two share the Mcq body and the grading section.
-->
<script lang="ts">
    import DifficultyBar from "./DifficultyBar.svelte";
    import {
        type AnswerResult,
        type Concept,
        type Node,
        type Problem,
        type Rating,
    } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import Mcq from "./Mcq.svelte";
    import ReviewCard from "./ReviewCard.svelte";
    import ScaffoldPicker from "./ScaffoldPicker.svelte";
    import Seam from "./Seam.svelte";

    export let root: Node;
    export let concept: Concept;
    export let conceptId: string;
    export let problem: Problem | null;
    export let answer: (rating: Rating) => Promise<AnswerResult>;
    export let onDone: (result: AnswerResult) => void;
    export let onError: (err: unknown) => void = () => {};

    let located = false;
    let answered = false;

    // Fresh card, fresh run.
    $: if (conceptId) {
        located = false;
        answered = false;
    }

    $: prompt = problem ? problem.prompt.trim() : "";
    $: locatePrompt = prompt || concept.title.trim() || "Locate this concept.";
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge state="hierarchy" />
    </svelte:fragment>

    <div class="sc-sec">
        <p class="step amber">Step ① · Locate in hierarchy</p>
        <p class="prompt">{locatePrompt}</p>
        <ScaffoldPicker {root} {conceptId} onComplete={() => (located = true)} />
    </div>

    {#if located}
        <Seam label="then identify · same screen" />
        <Mcq
            {problem}
            {concept}
            showPrompt={false}
            pickStep="②"
            pickTitle="Pick the answer"
            pickAmber
            checkedStep="③"
            onAnswered={() => (answered = true)}
        />

        {#if answered}
            <Seam label="same screen · now rate the card" />
            <div class="sc-sec sc-sec--grade">
                <p class="step">Step ④ · Rate difficulty</p>
                <DifficultyBar {answer} {onDone} {onError} />
            </div>
        {/if}
    {/if}
</ReviewCard>

<style lang="scss">
    @use "./sr-tokens" as srt;

    .step {
        @include srt.step;
    }
    .step.amber {
        --step-color: var(--sr-stage-guided-deep);
    }
    .prompt {
        margin: 9px 0 11px;
        font-size: 12px;
        line-height: 1.45;
        color: var(--sr-ink);
    }
</style>
