<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { Problem } from "./lib";

    export let problem: Problem;
    export let onChange: () => void;

    const LETTERS = ["A", "B", "C", "D"];
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
    <legend class="sr-only">Answer choices; select the correct one</legend>
    {#each [0, 1, 2, 3] as i (i)}
        <div class="choice" class:correct={problem.correctIndex === i}>
            <span class="letter" aria-hidden="true">{LETTERS[i]}</span>
            <label class="sr-only" for={`sr-choice-${problem.id}-${i}`}>
                Choice {LETTERS[i]}
            </label>
            <input
                id={`sr-choice-${problem.id}-${i}`}
                class="choice-input"
                bind:value={problem.choices[i]}
                on:input={onChoiceInput}
                placeholder={`Choice ${LETTERS[i]}`}
            />
            <input
                class="mark"
                type="radio"
                name={groupName}
                checked={problem.correctIndex === i}
                on:change={() => setCorrect(i)}
                title="Mark correct"
                aria-label={`Mark choice ${LETTERS[i]} correct`}
            />
        </div>
    {/each}
</fieldset>

<style lang="scss">
    .choices {
        border: none;
        margin: 0;
        padding: 0;
        display: grid;
        gap: 0.3rem;
    }
    .choice {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        padding: 0.1rem 0.1rem;
    }

    // The answer-sheet bubble: a mono letter that fills marigold when correct.
    .letter {
        flex-shrink: 0;
        width: 1.5rem;
        height: 1.5rem;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-family: var(--sr-mono);
        font-size: 0.78rem;
        font-weight: 600;
        color: var(--sr-ink-3);
        background: var(--sr-panel);
        border: 1px solid var(--sr-line-2);
        border-radius: 5px;
    }
    .choice.correct .letter {
        color: var(--sr-signal-ink);
        background: var(--sr-signal-weak);
        border-color: var(--sr-signal-line);
    }
    :global(.night-mode) .choice.correct .letter {
        color: var(--sr-signal);
    }

    // Each choice sits on its own answer line.
    .choice-input {
        flex: 1 1 auto;
        min-width: 0;
        border: none;
        border-bottom: 1px solid var(--sr-line-2);
        background: none;
        padding: 0.3rem 0.15rem;
        font: inherit;
        color: var(--sr-ink);
    }
    .choice-input::placeholder {
        color: var(--sr-ink-3);
    }
    .choice-input:focus {
        outline: none;
        border-bottom-color: var(--sr-signal);
    }
    .choice.correct .choice-input {
        border-bottom-color: var(--sr-signal-line);
    }

    .mark {
        flex-shrink: 0;
        width: 1.05rem;
        height: 1.05rem;
        accent-color: var(--sr-signal);
        cursor: pointer;
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
