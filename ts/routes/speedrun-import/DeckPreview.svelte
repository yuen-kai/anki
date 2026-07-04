<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { Concept, Hierarchy, Node } from "../speedrun-hierarchy/lib";
    import type { DeckCounts } from "./lib";

    const {
        hierarchy,
        summary,
        busy,
        onOpenBuilder,
        onBackToChat,
        onDiscard,
    }: {
        hierarchy: Hierarchy;
        summary: DeckCounts;
        busy: boolean;
        onOpenBuilder: () => void;
        onBackToChat: () => void;
        onDiscard: () => void;
    } = $props();

    // The generated blob carries a per-concept `source` for traceability that
    // the shared authoring type does not model; read it defensively.
    function sourceOf(concept: Concept): string {
        return (concept as Concept & { source?: string }).source ?? "";
    }
</script>

<div class="preview">
    <header class="head">
        <h2>{hierarchy.root.title}</h2>
        <p class="counts">
            {summary.groups} groups · {summary.topics} topics · {summary.concepts} concepts
        </p>
    </header>

    <div class="tree">
        {#each hierarchy.root.concepts as concept (concept.id)}
            {@render conceptView(concept)}
        {/each}
        {#each hierarchy.root.children as child (child.id)}
            {@render nodeView(child, 0)}
        {/each}
    </div>

    <div class="actions">
        <button class="link" onclick={onDiscard} disabled={busy}>Discard</button>
        <div class="right">
            <button onclick={onBackToChat} disabled={busy}>Back to chat</button>
            <button class="primary" onclick={onOpenBuilder} disabled={busy}>
                Open in builder
            </button>
        </div>
    </div>
</div>

{#snippet nodeView(node: Node, depth: number)}
    <section class="node" style:--depth={depth}>
        <h3 class="node-title">{node.title}</h3>
        {#each node.concepts as concept (concept.id)}
            {@render conceptView(concept)}
        {/each}
        {#each node.children as child (child.id)}
            {@render nodeView(child, depth + 1)}
        {/each}
    </section>
{/snippet}

{#snippet conceptView(concept: Concept)}
    <article class="concept">
        <div class="concept-head">
            <h4>{concept.title}</h4>
            {#if sourceOf(concept)}
                <span class="source">source: {sourceOf(concept)}</span>
            {/if}
        </div>
        {#if concept.content}
            <p class="content">{concept.content}</p>
        {/if}
        {#each concept.problems as problem (problem.id)}
            <div class="problem">
                <p class="prompt">{problem.prompt}</p>
                <ul class="choices">
                    {#each problem.choices as choice, index (index)}
                        <li class:correct={index === problem.correctIndex}>
                            <span class="mark" aria-hidden="true">
                                {index === problem.correctIndex ? "✓" : ""}
                            </span>
                            <span class="choice-text">{choice}</span>
                        </li>
                    {/each}
                </ul>
            </div>
        {/each}
    </article>
{/snippet}

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    .preview {
        @include sr.tokens;

        font-family: var(--sr-sans);
        color: var(--sr-ink);
    }
    .head {
        margin-bottom: 1rem;
    }
    h2 {
        margin: 0 0 0.25rem;
        font-size: 1.25rem;
        font-weight: 600;
    }
    .counts {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 0.72rem;
        letter-spacing: var(--sr-track-tag);
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .tree {
        max-height: 52vh;
        overflow-y: auto;
        padding-right: 0.25rem;
    }
    .node {
        margin-left: calc(var(--depth) * 0.9rem);
        padding-left: 0.75rem;
        border-left: 2px solid var(--sr-line-connector);
        margin-bottom: 0.75rem;
    }
    .node-title {
        margin: 0.5rem 0;
        font-size: 0.95rem;
        font-weight: 600;
    }
    .concept {
        border: 1px solid var(--sr-line);
        background: var(--sr-white);
        border-radius: var(--sr-radius-tile);
        padding: 0.75rem 0.85rem;
        margin: 0.5rem 0;
    }
    .concept-head {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 0.75rem;
    }
    h4 {
        margin: 0;
        font-size: 0.95rem;
        font-weight: 600;
    }
    .source {
        flex-shrink: 0;
        font-family: var(--sr-mono);
        font-size: 0.65rem;
        letter-spacing: 0.02em;
        color: var(--sr-faint);
    }
    .content {
        margin: 0.5rem 0 0.75rem;
        color: var(--sr-ink-2);
        line-height: 1.5;
        white-space: pre-wrap;
    }
    .problem {
        border-top: 1px solid var(--sr-line-2);
        padding-top: 0.6rem;
        margin-top: 0.6rem;
    }
    .prompt {
        margin: 0 0 0.5rem;
        font-weight: 540;
    }
    .choices {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
    }
    .choices li {
        display: flex;
        gap: 0.5rem;
        align-items: baseline;
        padding: 0.35rem 0.55rem;
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-chip);
        background: var(--sr-tile);
    }
    .choices li.correct {
        background: var(--sr-correct-bg);
        border-color: var(--sr-correct-line);
        color: var(--sr-correct-ink);
    }
    .mark {
        width: 0.9rem;
        flex-shrink: 0;
        color: var(--sr-stage-solo-deep);
        font-weight: 700;
    }
    .actions {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-top: 1.25rem;
        gap: 0.75rem;
    }
    .right {
        display: flex;
        gap: 0.6rem;
    }
    button {
        font: inherit;
        cursor: pointer;
        padding: 0.6rem 1rem;
        border-radius: var(--sr-radius-control);
        border: 1px solid var(--sr-line-strong);
        background: var(--sr-white);
        color: var(--sr-ink);
        font-weight: 560;
    }
    .primary {
        background: var(--sr-signal);
        border-color: var(--sr-signal);
        color: var(--sr-white);
    }
    button:disabled {
        opacity: 0.5;
        cursor: default;
    }
    .link {
        border: none;
        background: none;
        padding: 0;
        color: var(--sr-ink-2);
        text-decoration: underline;
        font-weight: 500;
    }
    :focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
    }
</style>
