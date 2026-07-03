<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Solo (also Guided when a concept sits on the root and has nothing to locate):
the question straight, its checked result disclosed below via the seam, then the
grading step. Shares the Mcq body and grading with Guided.
-->
<script lang="ts">
    import DifficultyBar from "./DifficultyBar.svelte";
    import { type AnswerResult, type Concept, type Problem, type Rating } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import Mcq from "./Mcq.svelte";
    import ReviewCard from "./ReviewCard.svelte";
    import Seam from "./Seam.svelte";

    export let concept: Concept;
    // Null when the concept authored no problems; Mcq falls back to a recall.
    export let problem: Problem | null;
    export let state: string;
    export let answer: (rating: Rating) => Promise<AnswerResult>;
    export let onDone: (result: AnswerResult) => void;
    export let onError: (err: unknown) => void = () => {};

    let answered = false;

    // Reset for each new card.
    $: if (problem || concept) {
        answered = false;
    }
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge {state} />
    </svelte:fragment>

    <Mcq
        {problem}
        {concept}
        showPrompt
        pickStep="①"
        pickTitle="The question"
        checkedStep="②"
        onAnswered={() => (answered = true)}
    />

    {#if answered}
        <Seam label="same screen · now rate the card" />
        <div class="sc-sec sc-sec--grade">
            <p class="step">Step ③ · Rate difficulty</p>
            <DifficultyBar {answer} {onDone} {onError} />
        </div>
    {/if}
</ReviewCard>

<style lang="scss">
    @use "./sr-tokens" as srt;

    .step {
        @include srt.step;
    }
</style>
