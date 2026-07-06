<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import MediaImage from "../speedrun-review/MediaImage.svelte";
    import type { Problem } from "./lib";

    export let problem: Problem;
    export let onChange: () => void;

    const groupName = `sr-correct-${problem.id}`;

    function setCorrect(index: number): void {
        problem.correctIndex = index;
        onChange();
    }

    function onChoiceInput(): void {
        onChange();
    }
</script>

<fieldset class="choices">
    <legend class="sr-only">Answer choices; mark the correct one</legend>
    {#each [0, 1, 2, 3] as i (i)}
        {@const correct = problem.correctIndex === i}
        {@const choiceImage = problem.choiceImages?.[i] ?? undefined}
        <div class="choice" class:correct>
            <input
                class="mark sr-only"
                id={`${groupName}-${i}`}
                type="radio"
                name={groupName}
                checked={correct}
                on:change={() => setCorrect(i)}
            />
            <label
                class="bubble"
                for={`${groupName}-${i}`}
                aria-label={`Mark answer ${i + 1} correct`}
            >
                {#if correct}✓{/if}
            </label>
            <input
                class="text"
                bind:value={problem.choices[i]}
                on:input={onChoiceInput}
                placeholder={`Answer ${i + 1}`}
                aria-label={`Answer ${i + 1}`}
            />
            {#if choiceImage}
                <div class="fig">
                    <MediaImage
                        filename={choiceImage}
                        alt={`Answer ${i + 1} figure`}
                        compact
                    />
                </div>
            {/if}
        </div>
    {/each}
</fieldset>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .choices {
        border: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    // The answer row: the shared MCQ primitive, tightened to the builder frame's
    // resting tile.
    .choice {
        @include syn.answer;

        background: var(--sr-tile);
        gap: 9px;
        flex-wrap: wrap;
        padding: 8px 11px;
        border-radius: var(--sr-radius-sm);
        font-size: 11.5px;
    }
    // A choice's figure drops onto its own full-width line beneath the answer
    // text; the flex row-gap already spaces it from the row above.
    .fig {
        flex-basis: 100%;
    }
    .choice.correct {
        @include syn.answer-correct;

        padding: 8px 11px;
        border-radius: var(--sr-radius-sm);
    }

    // The mark control is an accessible radio, drawn as the answer bubble: an
    // empty ring, or a filled coral-green check once correct.
    .bubble {
        @include syn.radio;

        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-family: var(--sr-mono);
        font-weight: 700;
        font-size: 9px;
        color: transparent;
    }
    .choice.correct .bubble {
        border: none;
        background: var(--sr-stage-solo);
        color: #fff;
    }
    .mark:focus-visible + .bubble {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
    }

    // Each choice sits on its own editable answer line.
    .text {
        flex: 1 1 auto;
        min-width: 0;
        border: none;
        background: none;
        padding: 0;
        font-family: var(--sr-sans);
        font-size: 11.5px;
        color: var(--sr-ink-2);
        caret-color: var(--sr-signal);
    }
    .choice.correct .text {
        color: var(--sr-stage-solo-deep);
        font-weight: 600;
    }
    .text::placeholder {
        color: var(--sr-faint);
    }

    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
    }
</style>
