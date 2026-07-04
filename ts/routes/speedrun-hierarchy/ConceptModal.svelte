<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The concept editor pane: a concept's title, its author-written description, and
its practice problems (each a prompt plus four answer choices with the correct
one marked). Rendered inline as the builder's third panel, and inline beneath
the concepts list when the panel runs self-contained (the demo).
-->
<script lang="ts">
    import ChoiceEditor from "./ChoiceEditor.svelte";
    import { type Concept, newProblem } from "./lib";

    export let concept: Concept;
    export let onChange: () => void;

    function field(): void {
        onChange();
    }

    function addProblem(): void {
        concept.problems = [...concept.problems, newProblem()];
        onChange();
    }

    function removeProblem(id: string): void {
        concept.problems = concept.problems.filter((problem) => problem.id !== id);
        onChange();
    }
</script>

<div class="editor3">
    <p class="label">Concept title</p>
    <input
        class="title"
        bind:value={concept.title}
        on:input={field}
        placeholder="Concept title"
        aria-label="Concept title"
    />

    <p class="label spaced">Content description</p>
    <textarea
        class="content"
        rows="3"
        bind:value={concept.content}
        on:input={field}
        placeholder="Concept description"
        aria-label="Content description"
    ></textarea>

    <div class="problems-head">
        <span class="label">Practice problems</span>
        <span class="hint">4 choices</span>
    </div>

    {#each concept.problems as problem, i (problem.id)}
        <div class="problem">
            <div class="problem-head">
                <input
                    class="prompt"
                    bind:value={problem.prompt}
                    on:input={field}
                    placeholder="Question prompt"
                    aria-label={`Problem ${i + 1} prompt`}
                />
                <button
                    class="del"
                    on:click={() => removeProblem(problem.id)}
                    aria-label="Delete problem"
                >
                    ×
                </button>
            </div>
            <ChoiceEditor {problem} onChange={field} />
        </div>
    {/each}

    <button class="affordance" on:click={addProblem}>
        <span class="plus" aria-hidden="true">+</span>
        Add practice problem
    </button>
</div>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .editor3 {
        display: flex;
        flex-direction: column;
    }

    .label {
        margin: 0 0 6px;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 500;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .label.spaced {
        margin-top: 16px;
    }

    .title {
        @include syn.input;
        @include syn.input-title;
    }

    .content {
        @include syn.textarea;

        padding: 12px 13px;
        line-height: 1.55;
        resize: vertical;
        min-height: 3.5rem;
        caret-color: var(--sr-signal);
    }
    .title::placeholder,
    .content::placeholder {
        color: var(--sr-faint);
    }

    .problems-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
        margin: 18px 0 8px;
    }
    .problems-head .label {
        margin: 0;
    }
    .hint {
        font-family: var(--sr-mono);
        font-size: 9px;
        font-weight: 500;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--sr-faint);
    }

    .problem {
        background: var(--sr-white);
        border: 1px solid var(--sr-line-strong);
        border-radius: 12px;
        padding: 13px 14px;
    }
    .problem + .problem {
        margin-top: 8px;
    }

    .problem-head {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 10px;
    }
    // The prompt reads as running text but stays editable, with a coral caret.
    .prompt {
        flex: 1 1 auto;
        min-width: 0;
        border: none;
        background: none;
        padding: 0;
        font-family: var(--sr-sans);
        font-size: 12.5px;
        font-weight: 600;
        line-height: 1.4;
        color: var(--sr-ink);
        caret-color: var(--sr-signal);
    }
    .prompt::placeholder {
        color: var(--sr-faint);
        font-weight: 500;
    }
    .del {
        flex-shrink: 0;
        width: 1.6rem;
        height: 1.6rem;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-white);
        color: var(--sr-ink-3);
        border-radius: 7px;
        font-size: 1rem;
        line-height: 1;
        cursor: pointer;
        opacity: 0;
        transition: opacity 0.12s ease;
    }
    .problem:hover .del,
    .problem:focus-within .del {
        opacity: 1;
    }
    .del:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }

    .affordance {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        width: fit-content;
        margin-top: 8px;
        border: 1.5px dashed var(--sr-line-dashed);
        border-radius: 11px;
        background: none;
        padding: 10px 14px;
        font-family: var(--sr-sans);
        font-size: 12px;
        font-weight: 600;
        color: var(--sr-ink-3);
        cursor: pointer;
    }
    .affordance:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }
    .affordance .plus {
        font-size: 1em;
        line-height: 1;
    }

    @media (prefers-reduced-motion: reduce) {
        .del {
            transition: none;
        }
    }
</style>
