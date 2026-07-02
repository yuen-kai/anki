<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Developer demo of the bespoke Speedrun screens. It reuses the real
speedrun-review study components and the speedrun-hierarchy authoring
components, driving them with the inline mock data in demo-fixtures so every
screen, state, and animation can be walked without a collection.

Nothing here touches the backend. The study RPCs are never called. The
authoring components run against a private cloned hierarchy through a demo
TreeContext whose change() only bumps a local revision, so no autosave is ever
scheduled and no speedrun* RPC fires: no deck is created, saved, or deleted.
-->
<script lang="ts">
    import { writable } from "svelte/store";

    import ChoiceEditor from "../speedrun-hierarchy/ChoiceEditor.svelte";
    import ConceptsPanel from "../speedrun-hierarchy/ConceptsPanel.svelte";
    import HierarchyTree from "../speedrun-hierarchy/HierarchyTree.svelte";
    import {
        findNode,
        type Hierarchy,
        isLeaf,
        type Node,
        type PulseState,
        setTreeContext,
        type TreeContext,
    } from "../speedrun-hierarchy/lib";
    import ConceptLearn from "../speedrun-review/ConceptLearn.svelte";
    import { findConcept, type Rating, rotateProblem } from "../speedrun-review/lib";
    import NewTopicIntro from "../speedrun-review/NewTopicIntro.svelte";
    import PracticeRecall from "../speedrun-review/PracticeRecall.svelte";
    import ProblemCard from "../speedrun-review/ProblemCard.svelte";
    import ScaffoldPicker from "../speedrun-review/ScaffoldPicker.svelte";
    import UpgradeAnimation from "../speedrun-review/UpgradeAnimation.svelte";
    import {
        cloneHierarchy,
        DEMO_CONCEPT_ID,
        DEMO_DECKS,
        DEMO_HIERARCHY,
        DEMO_LEARNING_CONCEPTS,
        DEMO_TOPIC_NODE_ID,
    } from "./demo-fixtures";

    type SceneId =
        | "create-deck"
        | "hierarchy"
        | "concept"
        | "intro"
        | "learning"
        | "upgrade-lp"
        | "practicing"
        | "applying"
        | "upgrade-am"
        | "mastering"
        | "done";

    interface Scene {
        id: SceneId;
        label: string;
        caption: string;
    }

    // Authoring first, then the study flow, so the numbered rail reads as the
    // real ordered sequence: build a deck, then study it, top to bottom.
    const scenes: Scene[] = [
        {
            id: "create-deck",
            label: "Create deck",
            caption: "The deck ledger. Create a deck to start a hierarchy.",
        },
        {
            id: "hierarchy",
            label: "Hierarchy editor",
            caption: "Build the topic tree. Select a leaf to add concepts.",
        },
        {
            id: "concept",
            label: "Concept editor",
            caption: "Write a concept and mark the correct choice on each problem.",
        },
        {
            id: "intro",
            label: "New topic",
            caption: "New topic reveal. The path lights down the tree to the leaf.",
        },
        {
            id: "learning",
            label: "Learning",
            caption:
                "Two contrasting cases, note the shared idea, then reveal the concept.",
        },
        {
            id: "upgrade-lp",
            label: "Learning to practicing",
            caption: "Stage change from learning to practicing.",
        },
        {
            id: "practicing",
            label: "Practicing",
            caption: "Free-response recall, reveal, then rate the difficulty.",
        },
        {
            id: "applying",
            label: "Applying",
            caption: "Place the concept in the tree, then work the problem.",
        },
        {
            id: "upgrade-am",
            label: "Applying to mastering",
            caption: "Stage change from applying to mastering.",
        },
        {
            id: "mastering",
            label: "Mastering",
            caption: "Work the problem with the scaffold gone.",
        },
        {
            id: "done",
            label: "Session complete",
            caption: "Nothing due. The finished session state.",
        },
    ];

    // The single concept the Practicing/Applying/Mastering scenes drill, so one
    // idea is seen climbing the ladder. The fixture guarantees it exists.
    const featured =
        findConcept(DEMO_HIERARCHY.root, DEMO_CONCEPT_ID) ?? DEMO_LEARNING_CONCEPTS[0];

    // Authoring scenes reuse the real deck-editor components. They edit a private
    // clone of the hierarchy through this demo TreeContext: change() only bumps a
    // local revision so derived views refresh, and it never schedules an
    // autosave, so no speedrun* RPC is sent and no real deck is ever written.
    const selectedId = writable<string | null>(null);
    const pulseState = writable<PulseState | null>(null);
    let authoring: Hierarchy = cloneHierarchy();
    let authoringRev = 0;
    let pulseSeq = 0;

    const treeContext: TreeContext = {
        change: () => {
            authoringRev += 1;
        },
        pulse: (ids) => {
            pulseSeq += 1;
            pulseState.set({ ids, seq: pulseSeq });
        },
        select: (id) => selectedId.set(id),
        selectedId,
        pulseState,
    };
    setTreeContext(treeContext);

    const animated = new Set<SceneId>(["intro", "upgrade-lp", "upgrade-am"]);
    const interactive = new Set<SceneId>([
        "hierarchy",
        "concept",
        "learning",
        "practicing",
        "applying",
        "mastering",
    ]);

    let sceneIndex = 0;
    // Bumped on every scene entry and every replay/reset; used as the {#key} for
    // the stage so animated scenes replay and interactive cards reset cleanly.
    let stageKey = 0;

    // Per-scene transient state.
    let learnIndex = 0;
    let applyPhase: "scaffold" | "problem" = "scaffold";
    let applyRotation = 0;
    let masterRotation = 0;

    // The contextual toolbar action: replay an animation, reset an interactive
    // card, or nothing on the static scenes.
    function actionFor(id: SceneId): string | null {
        if (animated.has(id)) {
            return "Replay";
        }
        if (interactive.has(id)) {
            return "Reset";
        }
        return null;
    }

    // The selected node, but only when it is a leaf (branches show no concepts).
    // Passing authoringRev in makes in-place edits (add/remove child) refresh it,
    // mirroring the real editor without any of its save plumbing.
    function pickLeaf(root: Node, id: string | null, _rev: number): Node | null {
        const node = findNode(root, id);
        return node && isLeaf(node) ? node : null;
    }

    $: scene = scenes[sceneIndex];
    $: learnConcept = DEMO_LEARNING_CONCEPTS[learnIndex];
    $: applyProblem = rotateProblem(featured.problems, applyRotation);
    $: masterProblem = rotateProblem(featured.problems, masterRotation);
    $: sceneAction = actionFor(scene.id);
    $: authoringLeaf = pickLeaf(authoring.root, $selectedId, authoringRev);
    $: authoringConcept = findConcept(authoring.root, DEMO_CONCEPT_ID);

    function enter(index: number): void {
        sceneIndex = index;
        learnIndex = 0;
        applyPhase = "scaffold";
        applyRotation = 0;
        masterRotation = 0;
        // Authoring scenes edit a private clone; rebuild it on every entry so
        // "Reset" and each visit start from the pristine fixture and never leak
        // edits into the shared study hierarchy. A leaf is pre-selected so the
        // concepts panel has content to show.
        authoring = cloneHierarchy();
        authoringRev = 0;
        selectedId.set(DEMO_TOPIC_NODE_ID);
        pulseState.set(null);
        stageKey += 1;
    }

    function goTo(id: SceneId): void {
        const index = scenes.findIndex((s) => s.id === id);
        if (index >= 0) {
            enter(index);
        }
    }

    function goPrev(): void {
        if (sceneIndex > 0) {
            enter(sceneIndex - 1);
        }
    }

    function goNext(): void {
        if (sceneIndex < scenes.length - 1) {
            enter(sceneIndex + 1);
        }
    }

    // Continue through the learning block: advance the concept counter, then hand
    // off to the learning-to-practicing upgrade once the block is taught.
    function onLearnNext(): void {
        if (learnIndex + 1 < DEMO_LEARNING_CONCEPTS.length) {
            learnIndex += 1;
        } else {
            goTo("upgrade-lp");
        }
    }

    function onPracticeRate(_rating: Rating): void {
        stageKey += 1;
    }

    function onScaffoldComplete(): void {
        applyPhase = "problem";
    }

    function onApplyRate(_rating: Rating): void {
        applyRotation += 1;
        applyPhase = "scaffold";
        stageKey += 1;
    }

    function onMasterRate(_rating: Rating): void {
        masterRotation += 1;
        stageKey += 1;
    }

    // Arrow keys step between scenes, but never while typing in a card's field.
    function onKeydown(event: KeyboardEvent): void {
        if (event.metaKey || event.ctrlKey || event.altKey) {
            return;
        }
        const target = event.target as HTMLElement | null;
        if (target && (target.tagName === "TEXTAREA" || target.tagName === "INPUT")) {
            return;
        }
        if (event.key === "ArrowLeft") {
            event.preventDefault();
            goPrev();
        } else if (event.key === "ArrowRight") {
            event.preventDefault();
            goNext();
        }
    }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="demo">
    <div class="shell">
        <aside class="rail">
            <div class="rail-head">
                <p class="eyebrow">Demo · mock data</p>
                <h1 class="rail-title">Study screens</h1>
                <p class="rail-note">
                    No collection and no cards. Every screen is driven by fixed data.
                </p>
            </div>
            <nav class="scene-list" aria-label="Demo scenes">
                {#each scenes as s, i (s.id)}
                    <button
                        class="scene"
                        class:active={i === sceneIndex}
                        aria-current={i === sceneIndex ? "true" : undefined}
                        on:click={() => enter(i)}
                    >
                        <span class="scene-index">
                            {String(i + 1).padStart(2, "0")}
                        </span>
                        <span class="scene-label">{s.label}</span>
                    </button>
                {/each}
            </nav>
        </aside>

        <main class="main">
            <div class="bar">
                <div class="stepper">
                    <button
                        class="ctl"
                        type="button"
                        on:click={goPrev}
                        disabled={sceneIndex === 0}
                    >
                        <span aria-hidden="true">←</span>
                        Prev
                    </button>
                    <span class="pos">{sceneIndex + 1} / {scenes.length}</span>
                    <button
                        class="ctl"
                        type="button"
                        on:click={goNext}
                        disabled={sceneIndex === scenes.length - 1}
                    >
                        Next <span aria-hidden="true">→</span>
                    </button>
                </div>
                {#if sceneAction}
                    <button
                        class="ctl action"
                        type="button"
                        on:click={() => enter(sceneIndex)}
                    >
                        {sceneAction}
                    </button>
                {/if}
            </div>

            <p class="caption">{scene.caption}</p>

            <div class="stage">
                {#key stageKey}
                    {#if scene.id === "create-deck"}
                        <div class="authoring decks-mock">
                            <header class="decks-head">
                                <h2 class="decks-title">Decks</h2>
                                <button
                                    class="decks-create"
                                    type="button"
                                    on:click={() => goTo("hierarchy")}
                                >
                                    Create deck
                                </button>
                            </header>
                            <table class="decks-grid">
                                <thead>
                                    <tr>
                                        <th scope="col">Name</th>
                                        <th scope="col" class="col-todo">To do</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each DEMO_DECKS as deck (deck.deckId)}
                                        <tr>
                                            <td>
                                                <button
                                                    class="deck-name"
                                                    type="button"
                                                    on:click={() => goTo("hierarchy")}
                                                >
                                                    {deck.name}
                                                </button>
                                            </td>
                                            <td class="col-todo">
                                                <span
                                                    class="deck-todo"
                                                    class:zero={deck.todo === 0}
                                                >
                                                    {deck.todo}
                                                </span>
                                            </td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        </div>
                    {:else if scene.id === "hierarchy"}
                        <div class="authoring editor-mock">
                            <header class="editor-head">
                                <p class="editor-crumb">
                                    Decks / {authoring.root.title || "Untitled deck"}
                                </p>
                                <h2 class="editor-title">Create hierarchy</h2>
                            </header>
                            <div class="editor-workspace">
                                <section
                                    class="editor-pane"
                                    aria-label="Deck structure"
                                >
                                    <p class="pane-label">Structure</p>
                                    <HierarchyTree root={authoring.root} />
                                </section>
                                <section class="editor-pane" aria-label="Concepts">
                                    {#if authoringLeaf}
                                        <ConceptsPanel node={authoringLeaf} />
                                    {:else}
                                        <div class="pane-placeholder">
                                            <p>Select a leaf topic to add concepts.</p>
                                        </div>
                                    {/if}
                                </section>
                            </div>
                        </div>
                    {:else if scene.id === "concept"}
                        {#if authoringConcept}
                            {@const concept = authoringConcept}
                            <div class="authoring concept-mock">
                                <p class="concept-eyebrow">Concept</p>
                                <label class="concept-field">
                                    <span class="concept-label">Title</span>
                                    <input
                                        class="concept-input"
                                        bind:value={concept.title}
                                        placeholder="Concept title"
                                    />
                                </label>
                                <label class="concept-field">
                                    <span class="concept-label">Description</span>
                                    <textarea
                                        class="concept-input concept-area"
                                        rows="3"
                                        bind:value={concept.content}
                                        placeholder="What this concept covers"
                                    ></textarea>
                                </label>
                                <div class="concept-problems">
                                    <h3 class="concept-problems-title">
                                        Practice problems
                                    </h3>
                                    <ol class="concept-problem-list">
                                        {#each concept.problems as problem, i (problem.id)}
                                            <li class="concept-problem">
                                                <div class="concept-problem-head">
                                                    <span class="concept-no">
                                                        {i + 1}
                                                    </span>
                                                    <input
                                                        class="concept-input"
                                                        bind:value={problem.prompt}
                                                        placeholder="Question prompt"
                                                        aria-label="Question prompt {i +
                                                            1}"
                                                    />
                                                </div>
                                                <ChoiceEditor
                                                    {problem}
                                                    onChange={() => {}}
                                                />
                                            </li>
                                        {/each}
                                    </ol>
                                </div>
                            </div>
                        {/if}
                    {:else if scene.id === "intro"}
                        <NewTopicIntro
                            root={DEMO_HIERARCHY.root}
                            topicNodeId={DEMO_TOPIC_NODE_ID}
                            onStart={() => goTo("learning")}
                        />
                    {:else if scene.id === "learning"}
                        <ConceptLearn
                            concept={learnConcept}
                            current={learnIndex + 1}
                            total={DEMO_LEARNING_CONCEPTS.length}
                            onNext={onLearnNext}
                        />
                    {:else if scene.id === "upgrade-lp"}
                        <UpgradeAnimation
                            from="learning"
                            to="practicing"
                            onDone={() => goTo("practicing")}
                        />
                    {:else if scene.id === "practicing"}
                        <PracticeRecall concept={featured} onRate={onPracticeRate} />
                    {:else if scene.id === "applying"}
                        {#if applyPhase === "scaffold"}
                            <ScaffoldPicker
                                root={DEMO_HIERARCHY.root}
                                conceptId={DEMO_CONCEPT_ID}
                                onComplete={onScaffoldComplete}
                            />
                        {:else}
                            <ProblemCard
                                concept={featured}
                                problem={applyProblem}
                                state="hierarchy"
                                onRate={onApplyRate}
                            />
                        {/if}
                    {:else if scene.id === "upgrade-am"}
                        <UpgradeAnimation
                            from="hierarchy"
                            to="mastering"
                            onDone={() => goTo("mastering")}
                        />
                    {:else if scene.id === "mastering"}
                        <ProblemCard
                            concept={featured}
                            problem={masterProblem}
                            state="mastering"
                            onRate={onMasterRate}
                        />
                    {:else if scene.id === "done"}
                        <div class="message">
                            <p class="done-eyebrow">Session complete</p>
                            <p class="message-title">Nothing due right now</p>
                            <p class="muted">
                                You have cleared every card the scheduler had ready.
                            </p>
                            <div class="actions">
                                <button
                                    class="btn primary"
                                    type="button"
                                    on:click={() => goTo("intro")}
                                >
                                    Back to overview
                                </button>
                                <button
                                    class="btn"
                                    type="button"
                                    on:click={() => goTo("intro")}
                                >
                                    All decks
                                </button>
                            </div>
                        </div>
                    {/if}
                {/key}
            </div>
        </main>
    </div>
</div>

<style lang="scss">
    @use "../speedrun-review/sr-tokens" as srt;
    @use "$lib/sass/speedrun-tokens" as sr;

    // The "score instrument" palette comes from the study route's shared partial
    // so the demo and the real study screen stay in lockstep (light + dark). The
    // reused components inherit --sr-* from .demo via the cascade.
    .demo {
        box-sizing: border-box;
        min-height: 100vh;
        background: var(--sr-paper);
        color: var(--sr-ink);
        font-family: var(--sr-sans);
        font-size: 15px;
        line-height: 1.55;
        letter-spacing: -0.003em;
        -webkit-font-smoothing: antialiased;

        @include srt.sr-tokens;
    }

    .shell {
        min-height: 100vh;
        display: grid;
        grid-template-columns: 15rem 1fr;
    }

    // Left rail: the numbered scene index.
    .rail {
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        padding: 1.75rem 1.25rem;
        border-right: 1px solid var(--sr-line);
        background: var(--sr-panel-2);
    }
    .rail-head {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }
    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .rail-title {
        margin: 0;
        font-size: 1.35rem;
        font-weight: 720;
        letter-spacing: -0.02em;
    }
    .rail-note {
        margin: 0;
        font-size: 0.82rem;
        color: var(--sr-ink-2);
        max-width: 22ch;
    }

    .scene-list {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }
    .scene {
        appearance: none;
        display: flex;
        align-items: center;
        gap: 0.6rem;
        width: 100%;
        text-align: left;
        padding: 0.5rem 0.6rem;
        border: none;
        border-left: 2px solid transparent;
        border-radius: 0 6px 6px 0;
        background: transparent;
        color: var(--sr-ink-2);
        font: inherit;
        cursor: pointer;
        transition:
            color 0.12s ease,
            background 0.12s ease;
    }
    .scene:hover {
        background: var(--sr-panel);
        color: var(--sr-ink);
    }
    .scene.active {
        background: var(--sr-panel);
        color: var(--sr-ink);
        border-left-color: var(--sr-signal);
    }
    .scene-index {
        font-family: var(--sr-mono);
        font-size: 0.72rem;
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink-3);
    }
    .scene.active .scene-index {
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .scene.active .scene-index {
        color: var(--sr-signal);
    }
    .scene-label {
        font-size: 0.9rem;
        font-weight: 520;
    }

    // Right pane: the toolbar, the caption, and the live stage.
    .main {
        display: flex;
        flex-direction: column;
        min-width: 0;
    }
    .bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        flex-wrap: wrap;
        padding: 1rem 1.5rem;
        border-bottom: 1px solid var(--sr-line);
    }
    .stepper {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    .ctl {
        appearance: none;
        display: inline-flex;
        align-items: center;
        gap: 0.3rem;
        padding: 0.4rem 0.75rem;
        border: 1px solid var(--sr-line-2);
        border-radius: 7px;
        background: var(--sr-panel);
        color: var(--sr-ink-2);
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        cursor: pointer;
        transition:
            color 0.12s ease,
            border-color 0.12s ease,
            background 0.12s ease;
    }
    .ctl:hover:not(:disabled) {
        color: var(--sr-ink);
        border-color: var(--sr-ink-3);
        background: var(--sr-panel-2);
    }
    .ctl:disabled {
        opacity: 0.45;
        cursor: default;
    }
    .ctl.action {
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
        background: var(--sr-signal-weak);
    }
    :global(.night-mode) .ctl.action {
        color: var(--sr-signal);
    }
    .pos {
        font-family: var(--sr-mono);
        font-size: 11px;
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink-3);
        padding: 0 0.25rem;
    }

    .caption {
        margin: 0;
        padding: 0.75rem 1.5rem 0;
        font-family: var(--sr-mono);
        font-size: 0.78rem;
        letter-spacing: 0.02em;
        color: var(--sr-ink-3);
    }

    .stage {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 1rem;
        padding: 2rem 1.5rem 3rem;
    }

    // Authoring scenes reuse the deck-editor components, which read the
    // neumorphic "chassis" palette. Re-declaring those --sr-* on this wrapper
    // paints the authoring screens on their own palette (same variable names,
    // scoped to this subtree) while the study scenes keep the score-instrument
    // palette from .demo. The mixin is placed last per its mixed-declarations
    // note.
    .authoring {
        box-sizing: border-box;
        width: 100%;
        margin: 0 auto;
        text-align: left;
        color: var(--sr-ink);
        font-family: var(--sr-sans);

        @include sr.tokens;
    }

    // Create deck: the decks ledger, ruled rows and mono tabular counts.
    .decks-mock {
        max-width: 46rem;
    }
    .decks-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding-bottom: 1rem;
        border-bottom: 2px solid var(--sr-ink);
    }
    .decks-title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 720;
        letter-spacing: -0.02em;
    }
    .decks-create {
        appearance: none;
        border: 1px solid var(--sr-signal-line);
        border-radius: 6px;
        padding: 0.5rem 0.9rem;
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
        font: inherit;
        font-weight: 560;
        cursor: pointer;
    }
    .decks-create:hover {
        filter: brightness(1.04);
    }
    .decks-grid {
        width: 100%;
        margin-top: 1.25rem;
        border-collapse: collapse;
    }
    .decks-grid th {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
        text-align: left;
        padding: 0 0.75rem 0.6rem;
        border-bottom: 1.5px solid var(--sr-ink);
    }
    .decks-grid td {
        padding: 0.3rem 0.75rem;
        border-bottom: 1px solid var(--sr-line);
    }
    .col-todo {
        width: 7rem;
        text-align: right;
    }
    .deck-name {
        display: block;
        width: 100%;
        padding: 0.5rem 0;
        border: none;
        background: none;
        color: var(--sr-ink);
        font: inherit;
        font-weight: 560;
        text-align: left;
        cursor: pointer;
    }
    .deck-name:hover {
        text-decoration: underline;
        text-underline-offset: 3px;
    }
    .deck-todo {
        font-family: var(--sr-mono);
        font-variant-numeric: tabular-nums;
        color: var(--sr-ink);
    }
    .deck-todo.zero {
        color: var(--sr-ink-3);
    }

    // Hierarchy editor: the breadcrumb header and two-pane workspace, matched to
    // the real editor so the reused tree and concepts panel sit in context.
    .editor-mock {
        max-width: 62rem;
    }
    .editor-head {
        padding-bottom: 0.9rem;
        border-bottom: 2px solid var(--sr-ink);
    }
    .editor-crumb {
        margin: 0 0 0.3rem;
        font-size: 0.82rem;
        color: var(--sr-ink-2);
    }
    .editor-title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 720;
        letter-spacing: -0.02em;
    }
    .editor-workspace {
        display: grid;
        grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
        gap: 1.25rem;
        margin-top: 1.5rem;
        align-items: start;
    }
    .editor-pane {
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        padding: 1rem 1.1rem 1.2rem;
    }
    .pane-label {
        margin: 0 0 0.6rem;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .pane-placeholder {
        color: var(--sr-ink-3);
        font-size: 0.9rem;
        padding: 1.5rem 0.25rem;
    }
    .pane-placeholder p {
        margin: 0;
    }

    // Concept editor: the concept fields plus the real four-choice problem
    // editor, laid out inline (no modal) so the demo chrome stays reachable.
    .concept-mock {
        max-width: 40rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        padding: 1.25rem;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 10px;
    }
    .concept-eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .concept-field {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }
    .concept-label {
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.13em;
        text-transform: uppercase;
        color: var(--sr-ink-3);
    }
    .concept-input {
        width: 100%;
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel-2);
        color: var(--sr-ink);
        border-radius: 6px;
        padding: 0.5rem 0.6rem;
        font: inherit;
    }
    .concept-input:focus {
        outline: none;
        border-color: var(--sr-signal-line);
        background: var(--sr-panel);
    }
    .concept-input::placeholder {
        color: var(--sr-ink-3);
    }
    .concept-area {
        resize: vertical;
        min-height: 3.5rem;
    }
    .concept-problems {
        border-top: 1px solid var(--sr-line);
        padding-top: 0.9rem;
    }
    .concept-problems-title {
        margin: 0 0 0.7rem;
        font-size: 0.95rem;
        font-weight: 640;
    }
    .concept-problem-list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .concept-problem {
        border: 1px solid var(--sr-line);
        border-radius: 8px;
        padding: 0.7rem;
        background: var(--sr-panel-2);
    }
    .concept-problem-head {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 0.6rem;
    }
    .concept-no {
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
    .concept-problem-head .concept-input {
        flex: 1 1 auto;
        min-width: 0;
    }

    // Finished state, mirroring the real orchestrator's copy.
    .message {
        max-width: 34rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        text-align: center;
    }
    .done-eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        color: var(--sr-signal-ink);
    }
    :global(.night-mode) .done-eyebrow {
        color: var(--sr-signal);
    }
    .muted {
        margin: 0;
        color: var(--sr-ink-3);
        font-family: var(--sr-mono);
        font-size: 0.8rem;
        letter-spacing: 0.04em;
    }
    .message-title {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 700;
        letter-spacing: -0.015em;
    }
    .actions {
        margin-top: 1rem;
        display: flex;
        gap: 0.6rem;
        flex-wrap: wrap;
        justify-content: center;
    }
    .btn {
        appearance: none;
        border: 1px solid var(--sr-line-2);
        border-radius: 8px;
        padding: 0.6rem 1.3rem;
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
    .btn.primary {
        background: var(--sr-signal);
        color: var(--sr-signal-ink);
        border-color: var(--sr-signal-line);
    }
    .btn.primary:hover {
        filter: brightness(1.04);
    }

    // Keyboard focus is always visible, on the chrome and the reused components.
    .demo :global(:focus-visible) {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 6px;
    }

    @media (prefers-reduced-motion: reduce) {
        .scene,
        .ctl {
            transition: none;
        }
    }

    @media (max-width: 48rem) {
        .shell {
            grid-template-columns: 1fr;
        }
        .editor-workspace {
            grid-template-columns: 1fr;
        }
        .rail {
            border-right: none;
            border-bottom: 1px solid var(--sr-line);
        }
        .scene-list {
            flex-direction: row;
            flex-wrap: nowrap;
            overflow-x: auto;
            gap: 0.4rem;
            padding-bottom: 0.25rem;
        }
        .scene {
            flex: 0 0 auto;
            border-left: none;
            border-bottom: 2px solid transparent;
            border-radius: 6px;
        }
        .scene.active {
            border-left-color: transparent;
            border-bottom-color: var(--sr-signal);
        }
        .scene-label {
            white-space: nowrap;
        }
    }
</style>
