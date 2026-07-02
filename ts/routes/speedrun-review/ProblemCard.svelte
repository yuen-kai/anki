<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import DifficultyBar from "./DifficultyBar.svelte";
    import { type Concept, type Problem, type Rating } from "./lib";
    import MasteryBadge from "./MasteryBadge.svelte";
    import ReviewCard from "./ReviewCard.svelte";

    export let concept: Concept;
    // Null when the concept authored no problems; we fall back to a plain recall.
    export let problem: Problem | null;
    export let state: string;
    export let onRate: (rating: Rating) => void;

    const LETTERS = ["A", "B", "C", "D"];

    let picked: number | null = null;
    let revealed = false;

    // Reset for each new card, whether it is an MCQ or the recall fallback.
    $: if (problem || concept) {
        picked = null;
        revealed = false;
    }

    $: hasKey = problem !== null && problem.correctIndex >= 0;
    $: content = concept.content.trim();

    function pick(index: number): void {
        if (picked === null) {
            picked = index;
        }
    }
</script>

<ReviewCard>
    <svelte:fragment slot="header">
        <MasteryBadge {state} />
    </svelte:fragment>

    {#if problem}
        <p class="prompt">{problem.prompt || "Untitled problem"}</p>
        <ul class="choices">
            {#each problem.choices as choice, i (i)}
                <li>
                    <button
                        class="choice"
                        class:correct={picked !== null &&
                            hasKey &&
                            i === problem.correctIndex}
                        class:wrong={picked === i &&
                            hasKey &&
                            i !== problem.correctIndex}
                        class:picked={picked === i}
                        on:click={() => pick(i)}
                        disabled={picked !== null}
                    >
                        <span class="letter">{LETTERS[i]}</span>
                        <span class="text">{choice || "(blank)"}</span>
                    </button>
                </li>
            {/each}
        </ul>
        {#if picked !== null && hasKey}
            <p class="verdict" class:ok={picked === problem.correctIndex}>
                {picked === problem.correctIndex ? "Correct" : "Not quite"}
            </p>
        {/if}
    {:else}
        <!-- No authored MCQ: grade the concept as a plain recall instead. -->
        <p class="eyebrow">Recall</p>
        <h1 class="cue">{concept.title.trim() || "Untitled concept"}</h1>
        {#if revealed}
            <div class="reveal">
                <p class="label">The concept</p>
                {#if content}
                    <p class="body">{content}</p>
                {:else}
                    <p class="body none">
                        No description was authored for this concept.
                    </p>
                {/if}
            </div>
        {/if}
    {/if}

    <svelte:fragment slot="footer">
        {#if problem}
            <DifficultyBar {onRate} active={picked !== null} />
        {:else if revealed}
            <DifficultyBar {onRate} />
        {:else}
            <button class="btn" on:click={() => (revealed = true)}>Show answer</button>
        {/if}
    </svelte:fragment>
</ReviewCard>

<style lang="scss">
    .prompt {
        margin: 0;
        font-size: 1.2rem;
        font-weight: 620;
        line-height: 1.35;
    }
    .choices {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    .choice {
        width: 100%;
        display: flex;
        align-items: center;
        gap: 0.7rem;
        padding: 0.65rem 0.75rem;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        background: var(--sr-panel);
        color: var(--sr-ink);
        font: inherit;
        text-align: left;
        cursor: pointer;
        transition:
            border-color 0.12s ease,
            background 0.12s ease;
    }
    .choice:hover:not(:disabled) {
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    .choice:disabled {
        cursor: default;
    }
    .letter {
        flex-shrink: 0;
        width: 1.6rem;
        height: 1.6rem;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-family: var(--sr-mono);
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--sr-ink-3);
        background: var(--sr-panel-2);
        border: 1px solid var(--sr-line-2);
        border-radius: 5px;
    }
    .text {
        min-width: 0;
    }
    // The authored-correct choice: filled marigold once an answer is locked in.
    .choice.correct {
        border-color: var(--sr-signal-line);
        background: var(--sr-signal-weak);
    }
    .choice.correct .letter {
        color: var(--sr-signal-ink);
        background: var(--sr-panel);
        border-color: var(--sr-signal-line);
    }
    :global(.night-mode) .choice.correct .letter {
        color: var(--sr-signal);
    }
    // A wrong pick reads as struck-through ink, no second alarm colour.
    .choice.wrong {
        border-color: var(--sr-line-2);
        opacity: 0.7;
    }
    .choice.wrong .text {
        text-decoration: line-through;
        color: var(--sr-ink-3);
    }
    .verdict {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 0.78rem;
        letter-spacing: 0.04em;
        color: var(--sr-ink-2);
    }
    .verdict.ok {
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .verdict.ok {
        color: var(--sr-signal);
    }

    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .cue {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 680;
        letter-spacing: -0.015em;
    }
    .reveal {
        border-top: 1px solid var(--sr-line);
        padding-top: 0.9rem;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    .label {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.13em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .body {
        margin: 0;
        line-height: 1.6;
        max-width: 42rem;
    }
    .body.none {
        color: var(--sr-ink-3);
        font-style: italic;
    }
    .btn {
        appearance: none;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        padding: 0.6rem 1.4rem;
        background: var(--sr-panel);
        color: var(--sr-ink);
        font: inherit;
        font-weight: 560;
        cursor: pointer;
    }
    .btn:hover {
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
</style>
