<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Level-up (individual, per-concept). One celebration card: a 4-node stepper
(Learn -> Practice -> Guided -> Solo) where every node up to the reached stage
fills in its own stage colour, the reached node ignites (a gold medal, expanding
ring halos, a glow), and restrained confetti drops from the top. Names the
concept and states in one line what the new stage entails. No XP, no levels.
The whole card is the advance control (click, Enter, or Space); a safety timer
advances on its own so a session never stalls.
-->
<script lang="ts">
    import { onDestroy, onMount } from "svelte";

    import {
        MASTERY_STAGES,
        stageBlurb,
        stageColor,
        stageLabel,
        stageRank,
    } from "./lib";

    // The concept that levelled up, plus the from/to internal states.
    export let concept: string;
    export let from: string;
    export let to: string;
    export let onDone: () => void;

    // The deep (readable-on-tint) variant of each stage colour, parallel to the
    // ladder; the reached node's label uses it. stageColor() only gives the base.
    const STAGE_DEEP = [
        "var(--sr-stage-learn-deep)",
        "var(--sr-stage-practice-deep)",
        "var(--sr-stage-guided-deep)",
        "var(--sr-stage-solo-deep)",
    ];

    $: reached = stageRank(to);
    $: title = concept.trim() || "This concept";
    $: meaning = stageBlurb(to);

    // done < reached < pending; done/reached wear their own stage colour.
    function nodeStatus(
        i: number,
        reachedIndex: number,
    ): "done" | "reached" | "pending" {
        if (i < reachedIndex) {
            return "done";
        }
        return i === reachedIndex ? "reached" : "pending";
    }

    $: nodes = MASTERY_STAGES.map((stage, i) => ({
        label: stage.label,
        status: nodeStatus(i, reached),
        color: stageColor(i),
        deep: STAGE_DEEP[i] ?? STAGE_DEEP[0],
        number: i + 1,
    }));

    // The connector leading into node k is filled in k's stage colour once the
    // path has reached k, otherwise it stays the grey track.
    function connectorColor(into: number): string {
        return into <= reached ? stageColor(into) : "var(--sr-track)";
    }

    const reduced =
        typeof window !== "undefined" &&
        window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    // Restrained confetti: five squares dropping from the top edge, one per
    // stage colour plus the coral accent. Values match the design canvas. None
    // when reduced motion is requested.
    const CONFETTI = reduced
        ? []
        : [
              {
                  left: 20,
                  size: 7,
                  color: "var(--sr-stage-guided)",
                  dur: 3,
                  delay: 0.1,
              },
              { left: 38, size: 6, color: "var(--sr-signal)", dur: 3.4, delay: 0.5 },
              {
                  left: 56,
                  size: 7,
                  color: "var(--sr-stage-practice)",
                  dur: 2.8,
                  delay: 0.3,
              },
              {
                  left: 70,
                  size: 6,
                  color: "var(--sr-stage-solo)",
                  dur: 3.2,
                  delay: 0.65,
              },
              {
                  left: 84,
                  size: 7,
                  color: "var(--sr-stage-guided)",
                  dur: 3,
                  delay: 0.8,
              },
          ];

    let fired = false;
    let timer: ReturnType<typeof setTimeout> | undefined;

    function finish(): void {
        if (fired) {
            return;
        }
        fired = true;
        if (timer) {
            clearTimeout(timer);
        }
        onDone();
    }

    function onKey(event: KeyboardEvent): void {
        if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            finish();
        }
    }

    // A safety auto-advance so a session never stalls on the moment, but long
    // enough to read it; the learner usually clicks or keys through first.
    onMount(() => {
        timer = setTimeout(finish, reduced ? 3200 : 6000);
    });
    onDestroy(() => {
        if (timer) {
            clearTimeout(timer);
        }
    });
</script>

<svelte:window on:keydown={onKey} />

<div class="wrap" class:reduced>
    <div class="card">
        {#each CONFETTI as bit, i (i)}
            <span
                class="confetti"
                aria-hidden="true"
                style="left: {bit.left}%; width: {bit.size}px; height: {bit.size}px; background: {bit.color}; animation-duration: {bit.dur}s; animation-delay: {bit.delay}s"
            ></span>
        {/each}

        <p class="eyebrow">Stage up</p>
        <h1 class="concept">{title}</h1>

        <div
            class="stepper"
            role="img"
            aria-label={`Leveled up from ${stageLabel(from)} to ${stageLabel(to)}`}
        >
            {#each nodes as node, i (i)}
                <div
                    class="node"
                    class:done={node.status === "done"}
                    class:reached={node.status === "reached"}
                    class:pending={node.status === "pending"}
                    style="--node: {node.color}"
                >
                    {#if node.status === "reached"}
                        <div class="dot-row">
                            <div class="ignite">
                                {#if !reduced}
                                    <span class="ring r1" aria-hidden="true"></span>
                                    <span class="ring r2" aria-hidden="true"></span>
                                {/if}
                                <span class="medal">{node.number}</span>
                            </div>
                        </div>
                        <span class="label" style="color: {node.deep}">
                            {node.label}
                        </span>
                    {:else}
                        <div class="dot-row">
                            <span class="dot">
                                {#if node.status === "done"}✓{:else}{node.number}{/if}
                            </span>
                        </div>
                        <span class="label">{node.label}</span>
                    {/if}
                </div>
                {#if i < nodes.length - 1}
                    <span
                        class="connector"
                        aria-hidden="true"
                        style="--conn: {connectorColor(i + 1)}"
                    ></span>
                {/if}
            {/each}
        </div>

        <p class="meaning">{meaning}</p>
    </div>

    <button
        class="advance"
        type="button"
        aria-label="Continue to next card"
        on:click={finish}
    ></button>
</div>

<style lang="scss">
    .wrap {
        position: relative;
        width: 100%;
        max-width: 37rem;
    }
    .card {
        position: relative;
        overflow: hidden;
        min-height: 340px;
        padding: 30px 24px 26px;
        background: var(--sr-panel);
        border-radius: var(--sr-radius-card);
        box-shadow: var(--sr-shadow-card);
        text-align: center;
        animation: sr-rise 380ms ease-out both;
    }

    .confetti {
        position: absolute;
        top: -8px;
        border-radius: 2px;
        animation-name: sr-confetti-drop;
        animation-timing-function: linear;
        animation-iteration-count: infinite;
    }

    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 700;
        letter-spacing: var(--sr-track-hero);
        text-transform: uppercase;
        color: var(--sr-signal);
    }
    .concept {
        margin: 12px 0 0;
        font-size: 19px;
        font-weight: 700;
        color: var(--sr-ink);
        line-height: 1.2;
    }

    .stepper {
        display: flex;
        align-items: flex-start;
        justify-content: center;
        margin: 30px 0 16px;
    }
    .node {
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 72px;
    }
    // A fixed-height dot row so every circle centre (28px dot or 48px halo)
    // lands on the same line; the connectors align to that centre.
    .dot-row {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 48px;
    }
    .dot {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        border-radius: 50%;
        box-sizing: border-box;
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 700;
    }
    .node.done .dot {
        background: var(--node);
        color: #fff;
    }
    .node.pending .dot {
        background: var(--sr-inset);
        border: 2px dashed var(--sr-line-dashed);
        color: var(--sr-line-dashed);
    }

    .ignite {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 48px;
        height: 48px;
    }
    .medal {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 38px;
        height: 38px;
        border-radius: 50%;
        background: var(--sr-grad-medal);
        color: #fff;
        font-family: var(--sr-mono);
        font-size: 15px;
        font-weight: 800;
        box-shadow: 0 4px 16px color-mix(in srgb, var(--node) 70%, transparent);
        animation: sr-medal-pop 2.4s ease-out infinite;
    }
    .ring {
        position: absolute;
        inset: 0;
        border-radius: 50%;
        box-sizing: border-box;
        animation: sr-ring-pop 2.4s ease-out infinite;
    }
    .ring.r1 {
        border: 2px solid color-mix(in srgb, var(--node) 55%, transparent);
    }
    .ring.r2 {
        border: 2px solid color-mix(in srgb, var(--node) 30%, transparent);
        animation-delay: 0.6s;
    }

    .label {
        margin-top: 7px;
        font-family: var(--sr-mono);
        font-size: 8px;
        font-weight: 600;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--node);
    }
    .node.reached .label {
        margin-top: 6px;
        font-weight: 700;
    }
    .node.pending .label {
        color: var(--sr-line-dashed);
    }

    .connector {
        width: 26px;
        height: 2px;
        margin-top: 23px;
        border-radius: 1px;
        background: var(--conn);
    }

    .meaning {
        max-width: 30rem;
        margin: 0 auto;
        font-size: 13px;
        line-height: 1.5;
        color: var(--sr-ink-2);
    }

    // The whole card advances the session; the transparent overlay keeps the
    // click target and native Enter/Space, and shows the focus ring on the card.
    .advance {
        position: absolute;
        inset: 0;
        margin: 0;
        padding: 0;
        border: none;
        background: transparent;
        cursor: pointer;
    }
    .advance:focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: var(--sr-radius-card);
    }

    @keyframes sr-rise {
        from {
            opacity: 0;
            transform: translateY(6px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    @keyframes sr-confetti-drop {
        from {
            transform: translateY(0) rotate(0);
        }
        to {
            transform: translateY(380px) rotate(150deg);
        }
    }
    @keyframes sr-ring-pop {
        0% {
            opacity: 1;
            transform: scale(0.6);
        }
        70% {
            opacity: 0;
        }
        100% {
            opacity: 0;
            transform: scale(1.4);
        }
    }
    @keyframes sr-medal-pop {
        0%,
        100% {
            transform: scale(1);
        }
        50% {
            transform: scale(1.05);
        }
    }

    // Reduced motion: keep the earned end-state, drop every moving part. The
    // rings and confetti are already not rendered; this covers the CSS-driven
    // pulses and the entrance for anyone the JS check missed.
    .reduced .card,
    .reduced .medal {
        animation: none;
    }
    @media (prefers-reduced-motion: reduce) {
        .card,
        .medal,
        .ring,
        .confetti {
            animation: none;
        }
    }

    @media (max-width: 26rem) {
        .node {
            width: 54px;
        }
        .connector {
            width: 14px;
        }
    }
</style>
