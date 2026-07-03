<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The question body shared by Guided and Solo: the pick step (four choices), then
— disclosed below via the seam on the same card — the checked step (the correct
answer and the concept's explanation). A wrong pick marks the chosen option ✕,
drops an inline correction under it, and reveals the correct option ✓ (the 2j
flow). Step numbers/labels are configurable so Guided (locate → pick → checked)
and Solo (question → checked) both read right. With no authored problem it
degrades to a recall.
-->
<script lang="ts">
    import { type Concept, type Problem } from "./lib";
    import Seam from "./Seam.svelte";

    // Null when the concept authored no problems; we fall back to a recall.
    export let problem: Problem | null;
    // The concept behind the question; its description is the explanation.
    export let concept: Concept;
    // Fired once committed (a choice picked, or the recall shown), so the parent
    // can disclose the grading step below.
    export let onAnswered: () => void = () => {};
    // Solo shows the prompt above the choices; Guided showed it in the locate
    // step, so it passes showPrompt=false.
    export let showPrompt = true;
    // The circled step number + its uppercase label + tone: Solo is the neutral
    // "① · The question"; Guided is the amber "② · Pick the answer".
    export let pickStep = "①";
    export let pickTitle = "The question";
    export let pickAmber = false;
    export let checkedStep = "②";

    let picked: number | null = null;
    let revealed = false;

    // Reset for each new question (rotation) or concept.
    $: if (problem || concept) {
        picked = null;
        revealed = false;
    }

    $: hasKey =
        problem !== null &&
        problem.correctIndex >= 0 &&
        problem.correctIndex < problem.choices.length;
    $: correctIndex = hasKey && problem ? problem.correctIndex : -1;
    $: explanation = concept.content.trim();

    function pick(index: number): void {
        if (picked !== null) {
            return;
        }
        picked = index;
        onAnswered();
    }

    function reveal(): void {
        revealed = true;
        onAnswered();
    }
</script>

{#if problem}
    <div class="sc-sec">
        <p class="step" class:amber={pickAmber}>STEP {pickStep} · {pickTitle}</p>
        {#if showPrompt}
            <p class="prompt">{problem.prompt || "Untitled problem"}</p>
        {/if}
        <ul class="answers">
            {#each problem.choices as choice, i (i)}
                <li>
                    <button
                        class="answer"
                        class:correct={picked !== null && hasKey && i === correctIndex}
                        class:wrong={picked === i && hasKey && i !== correctIndex}
                        type="button"
                        on:click={() => pick(i)}
                        disabled={picked !== null}
                    >
                        {#if picked !== null && hasKey && i === correctIndex}
                            <span class="badge badge--correct" aria-hidden="true">
                                ✓
                            </span>
                        {:else if picked === i && hasKey && i !== correctIndex}
                            <span class="badge badge--wrong" aria-hidden="true">✕</span>
                        {:else}
                            <span class="radio" aria-hidden="true"></span>
                        {/if}
                        <span class="text">{choice || "(blank)"}</span>
                    </button>
                    {#if picked === i && hasKey && i !== correctIndex}
                        <p class="correction" role="status">
                            Not quite. The correct answer is marked below.
                        </p>
                    {/if}
                </li>
            {/each}
        </ul>
    </div>

    {#if picked !== null}
        <Seam label="after you answer · same screen" />
        <div class="sc-sec sc-sec--reveal">
            <p class="step green">STEP {checkedStep} · Checked</p>
            {#if hasKey && problem}
                <div class="answer correct static">
                    <span class="badge badge--correct" aria-hidden="true">✓</span>
                    <span class="text">
                        {problem.choices[correctIndex] || "(blank)"}
                    </span>
                </div>
            {/if}
            {#if explanation}
                <p class="explain">{explanation}</p>
            {:else}
                <p class="explain none">
                    No explanation was authored for this concept.
                </p>
            {/if}
        </div>
    {/if}
{:else}
    <div class="sc-sec">
        <p class="step">STEP {pickStep} · Recall</p>
        <h2 class="cue">{concept.title.trim() || "Untitled concept"}</h2>
        {#if !revealed}
            <button class="show" type="button" on:click={reveal}>Show answer</button>
        {/if}
    </div>
    {#if revealed}
        <Seam label="same screen" />
        <div class="sc-sec sc-sec--reveal">
            <p class="step green">Checked</p>
            {#if explanation}
                <p class="explain">{explanation}</p>
            {:else}
                <p class="explain none">
                    No description was authored for this concept.
                </p>
            {/if}
        </div>
    {/if}
{/if}

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;
    @use "./sr-tokens" as srt;

    .step {
        @include srt.step;
    }
    .step.amber {
        --step-color: var(--sr-stage-guided-deep);
    }
    .step.green {
        --step-color: var(--sr-stage-solo-deep);
    }
    .prompt {
        margin: 9px 0 12px;
        font-size: 12.5px;
        line-height: 1.45;
        color: var(--sr-ink);
    }
    .cue {
        margin: 9px 0 12px;
        font-size: 18px;
        font-weight: 700;
        letter-spacing: var(--sr-tighten);
        color: var(--sr-ink);
    }

    .answers {
        list-style: none;
        margin: 9px 0 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 7px;
    }
    .answer {
        @include syn.answer;
    }
    button.answer {
        appearance: none;
        width: 100%;
        text-align: left;
        font-family: var(--sr-sans);
        cursor: pointer;
        transition:
            border-color 0.12s ease,
            background 0.12s ease;
    }
    button.answer:hover:not(:disabled) {
        border-color: var(--sr-line-dashed);
        background: var(--sr-tile);
    }
    button.answer:disabled {
        cursor: default;
    }
    .answer.correct {
        @include syn.answer-correct;
    }
    .answer.wrong {
        @include syn.answer-wrong;
    }
    // The correct-answer row repeated in the checked step is not interactive.
    .answer.static {
        margin-top: 0;
    }
    .radio {
        @include syn.radio;
    }
    .badge {
        @include syn.badge;
    }
    .badge--correct {
        background: var(--sr-stage-solo);
    }
    .badge--wrong {
        background: var(--sr-signal);
    }
    .text {
        min-width: 0;
    }

    .correction {
        @include syn.correction;
        margin: 7px 0 0;
    }
    .explain {
        @include syn.explain;
        margin: 10px 0 0;
    }
    .explain.none {
        color: var(--sr-ink-3);
        font-style: italic;
    }

    .show {
        appearance: none;
        margin-top: 4px;
        border: 1px solid var(--sr-line-strong);
        border-radius: var(--sr-radius-control);
        padding: 9px 16px;
        background: var(--sr-white);
        color: var(--sr-ink);
        font: inherit;
        font-weight: 600;
        font-size: 12px;
        cursor: pointer;
    }
    .show:hover {
        border-color: var(--sr-ink-3);
    }
</style>
