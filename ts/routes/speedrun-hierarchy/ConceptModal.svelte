<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";

    import ChoiceEditor from "./ChoiceEditor.svelte";
    import { type Concept, newProblem } from "./lib";

    export let concept: Concept;
    export let onChange: () => void;
    export let onClose: () => void;

    let titleRef: HTMLInputElement | undefined;

    onMount(() => {
        titleRef?.focus();
    });

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

    function onKeydown(event: KeyboardEvent): void {
        if (event.key === "Escape") {
            onClose();
        }
    }

    // Keep keyboard focus inside the open dialog (Tab / Shift+Tab wrap around).
    // Radio groups only expose their checked member to Tab, so filter to it.
    function trap(node: HTMLElement) {
        const FOCUSABLE =
            'a[href], button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

        function tabbable(): HTMLElement[] {
            return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
                (el) => {
                    if (el.offsetParent === null) {
                        return false;
                    }
                    if (
                        el instanceof HTMLInputElement &&
                        el.type === "radio" &&
                        el.name
                    ) {
                        const group = Array.from(
                            node.querySelectorAll<HTMLInputElement>(
                                `input[type="radio"][name="${CSS.escape(el.name)}"]`,
                            ),
                        );
                        return (
                            el === (group.find((radio) => radio.checked) ?? group[0])
                        );
                    }
                    return true;
                },
            );
        }

        function onTrapKeydown(event: KeyboardEvent): void {
            if (event.key !== "Tab") {
                return;
            }
            const items = tabbable();
            if (items.length === 0) {
                return;
            }
            const first = items[0];
            const last = items[items.length - 1];
            if (event.shiftKey && document.activeElement === first) {
                event.preventDefault();
                last.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
                event.preventDefault();
                first.focus();
            }
        }

        node.addEventListener("keydown", onTrapKeydown);
        return {
            destroy(): void {
                node.removeEventListener("keydown", onTrapKeydown);
            },
        };
    }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="overlay">
    <button
        class="backdrop"
        tabindex="-1"
        aria-label="Close concept"
        on:click={onClose}
    ></button>
    <div
        class="modal"
        use:trap
        role="dialog"
        aria-modal="true"
        aria-labelledby={`sr-modal-${concept.id}`}
    >
        <header class="modal-head">
            <h2 class="modal-title" id={`sr-modal-${concept.id}`}>Concept</h2>
            <button class="icon" on:click={onClose} aria-label="Close">×</button>
        </header>

        <div class="body">
            <label class="field">
                <span class="label">Title</span>
                <input
                    bind:this={titleRef}
                    class="text"
                    bind:value={concept.title}
                    on:input={field}
                    placeholder="Concept title"
                />
            </label>

            <label class="field">
                <span class="label">Description</span>
                <textarea
                    class="text area"
                    rows="3"
                    bind:value={concept.content}
                    on:input={field}
                    placeholder="What this concept covers"
                ></textarea>
            </label>

            <div class="problems">
                <div class="problems-head">
                    <h3 class="problems-title">Practice problems</h3>
                    <button class="add" on:click={addProblem}>Add problem</button>
                </div>

                {#if concept.problems.length === 0}
                    <p class="empty">No problems yet.</p>
                {:else}
                    <ol class="problem-list">
                        {#each concept.problems as problem, i (problem.id)}
                            <li class="problem">
                                <div class="problem-head">
                                    <span class="no">{i + 1}</span>
                                    <label
                                        class="sr-only"
                                        for={`sr-prompt-${problem.id}`}
                                    >
                                        Question prompt
                                    </label>
                                    <input
                                        id={`sr-prompt-${problem.id}`}
                                        class="text"
                                        bind:value={problem.prompt}
                                        on:input={field}
                                        placeholder="Question prompt"
                                    />
                                    <button
                                        class="icon"
                                        on:click={() => removeProblem(problem.id)}
                                        aria-label="Delete problem"
                                    >
                                        ×
                                    </button>
                                </div>
                                <ChoiceEditor {problem} onChange={field} />
                            </li>
                        {/each}
                    </ol>
                {/if}
            </div>
        </div>
    </div>
</div>

<style lang="scss">
    .overlay {
        position: fixed;
        inset: 0;
        z-index: 50;
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding: 4vh 1rem;
        overflow-y: auto;
        background: rgba(10, 12, 15, 0.5);
    }
    .backdrop {
        position: fixed;
        inset: 0;
        border: none;
        background: none;
        cursor: default;
    }

    .modal {
        position: relative;
        z-index: 1;
        display: flex;
        flex-direction: column;
        width: 100%;
        max-width: 40rem;
        max-height: 92vh;
        background: var(--sr-panel);
        color: var(--sr-ink);
        border: 1px solid var(--sr-line);
        border-radius: 10px;
        box-shadow: 0 18px 50px rgba(0, 0, 0, 0.3);
    }

    .modal-head {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 0.9rem 1.1rem;
        border-bottom: 1px solid var(--sr-line);
    }
    .modal-title {
        margin: 0;
        font-size: 1.05rem;
        font-weight: 660;
    }
    .icon {
        width: 2rem;
        height: 2rem;
        flex-shrink: 0;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        border-radius: 6px;
        font-size: 1.1rem;
        line-height: 1;
        cursor: pointer;
    }
    .icon:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }

    .body {
        overflow-y: auto;
        padding: 1.1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }
    .label {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.13em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .text {
        width: 100%;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel-2);
        color: var(--sr-ink);
        border-radius: 6px;
        padding: 0.5rem 0.6rem;
        font: inherit;
    }
    .text:focus {
        outline: none;
        border-color: var(--sr-signal-line);
        background: var(--sr-panel);
    }
    .text::placeholder {
        color: var(--sr-ink-3);
    }
    .area {
        resize: vertical;
        min-height: 3.5rem;
    }

    .problems {
        border-top: 1px solid var(--sr-line);
        padding-top: 0.9rem;
    }
    .problems-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
        margin-bottom: 0.7rem;
    }
    .problems-title {
        margin: 0;
        font-size: 0.95rem;
        font-weight: 640;
    }
    .add {
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        border-radius: 6px;
        padding: 0.4rem 0.7rem;
        font: inherit;
        font-size: 0.8rem;
        font-weight: 560;
        cursor: pointer;
        white-space: nowrap;
    }
    .add:hover {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
    }
    .empty {
        margin: 0;
        color: var(--sr-ink-3);
        font-size: 0.9rem;
    }

    .problem-list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
        counter-reset: none;
    }
    .problem {
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        padding: 0.7rem;
        background: var(--sr-panel-2);
    }
    .problem-head {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 0.6rem;
    }
    .no {
        flex-shrink: 0;
        width: 1.5rem;
        height: 1.5rem;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-family: var(--sr-mono);
        font-size: 0.75rem;
        color: var(--sr-ink-2);
        background: var(--sr-panel);
        border: 1px solid var(--sr-line-2);
        border-radius: 5px;
    }
    .problem-head .text {
        flex: 1 1 auto;
        min-width: 0;
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

    @media (max-width: 34rem) {
        .overlay {
            padding: 0;
        }
        .modal {
            max-height: 100vh;
            border-radius: 0;
            border: none;
        }
    }
</style>
