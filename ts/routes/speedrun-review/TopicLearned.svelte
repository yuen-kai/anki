<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Topic-learned (upgrade-together). Shown only when every concept in a topic
clears Learn at once (the topic-gated flip). One celebration card: names the
topic, states the shared stage change, and lists each concept, whose mastery
glyph flips to the reached stage in unison.
-->
<script lang="ts">
    import { onDestroy, onMount } from "svelte";

    import StageGlyph from "../speedrun-dashboard/StageGlyph.svelte";
    import { stageLabel, stageRank } from "./lib";

    export let topic: string;
    // The concepts that flipped together, in tree order.
    export let items: { title: string }[];
    export let from: string;
    export let to: string;
    export let onDone: () => void;

    $: fromIndex = stageRank(from);
    $: toIndex = stageRank(to);
    $: heading = topic.trim() || "Topic";
    $: countText =
        items.length === 1 ? "1 concept" : `all ${items.length} concepts together`;

    const reduced =
        typeof window !== "undefined" &&
        window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    // Restrained top-of-card spray, each square a stage colour, matching the
    // canvas. Empty (nothing rendered) when motion is reduced.
    const confetti = reduced
        ? []
        : [
              {
                  left: "22%",
                  size: 7,
                  color: "var(--sr-stage-practice)",
                  dur: "3s",
                  delay: "0.1s",
              },
              {
                  left: "44%",
                  size: 6,
                  color: "var(--sr-signal)",
                  dur: "3.4s",
                  delay: "0.5s",
              },
              {
                  left: "64%",
                  size: 7,
                  color: "var(--sr-stage-solo)",
                  dur: "2.8s",
                  delay: "0.3s",
              },
              {
                  left: "80%",
                  size: 6,
                  color: "var(--sr-stage-guided)",
                  dur: "3.2s",
                  delay: "0.7s",
              },
          ];

    // Reduced motion holds the after-state from the first paint; otherwise we
    // linger on "before" and flip every glyph together.
    let advanced = reduced;
    let fired = false;
    let holdTimer: ReturnType<typeof setTimeout> | undefined;
    let safety: ReturnType<typeof setTimeout> | undefined;

    function finish(): void {
        if (fired) {
            return;
        }
        fired = true;
        clearTimeout(holdTimer);
        clearTimeout(safety);
        onDone();
    }

    // Fallback for when focus is off the card overlay (which handles Enter/Space
    // natively as a real button).
    function onKey(event: KeyboardEvent): void {
        if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            finish();
        }
    }

    onMount(() => {
        if (!reduced) {
            holdTimer = setTimeout(() => (advanced = true), 520);
        }
        // Safety auto-advance so a session never stalls on the moment.
        safety = setTimeout(finish, reduced ? 3600 : 7000);
    });
    onDestroy(() => {
        clearTimeout(holdTimer);
        clearTimeout(safety);
    });
</script>

<svelte:window on:keydown={onKey} />

<div class="card" class:reduced>
    <div class="glow" aria-hidden="true"></div>
    {#each confetti as bit, i (i)}
        <span
            class="confetti"
            aria-hidden="true"
            style="left: {bit.left}; width: {bit.size}px; height: {bit.size}px; background: {bit.color}; animation-duration: {bit.dur}; animation-delay: {bit.delay}"
        ></span>
    {/each}

    <div class="content">
        <p class="eyebrow">Topic leveled up</p>
        <h1 class="topic">{heading}</h1>
        <p class="transition">
            <span class="from">{stageLabel(from)}</span>
            <span class="arrow" aria-hidden="true">→</span>
            <span class="to">{stageLabel(to)}</span>
            <span class="count">· {countText}</span>
        </p>

        <ul class="rows" class:advanced>
            {#each items as item, i (i)}
                <li class="row">
                    <span class="badge" aria-hidden="true">↑</span>
                    <span class="name">{item.title.trim() || "Untitled concept"}</span>
                    <span class="glyph">
                        <StageGlyph stage={advanced ? toIndex : fromIndex} size="lg" />
                    </span>
                </li>
            {/each}
        </ul>
    </div>

    <button
        class="overlay"
        type="button"
        aria-label="Continue to next card"
        on:click={finish}
    ></button>
</div>

<style lang="scss">
    .card {
        position: relative;
        overflow: hidden;
        box-sizing: border-box;
        width: 100%;
        max-width: 37rem;
        min-height: 340px;
        padding: 28px 24px 26px;
        background: var(--sr-panel);
        border-radius: var(--sr-radius-card);
        box-shadow: var(--sr-shadow-card);
        text-align: center;
    }

    .glow {
        position: absolute;
        inset: 0;
        pointer-events: none;
        background: radial-gradient(
            circle at 50% 34%,
            color-mix(in srgb, var(--sr-stage-practice) 12%, transparent),
            transparent 55%
        );
    }

    .confetti {
        position: absolute;
        top: -8px;
        border-radius: 2px;
        animation-name: sr-confetti-drop;
        animation-timing-function: linear;
        animation-iteration-count: infinite;
    }
    @keyframes sr-confetti-drop {
        0% {
            transform: translateY(-24px) rotate(0);
            opacity: 0;
        }
        12% {
            opacity: 1;
        }
        100% {
            transform: translateY(215px) rotate(340deg);
            opacity: 0;
        }
    }

    .content {
        position: relative;
        z-index: 1;
    }

    .eyebrow {
        margin: 0;
        font-family: var(--sr-mono);
        font-size: 11px;
        font-weight: 700;
        letter-spacing: var(--sr-track-hero);
        text-transform: uppercase;
        color: var(--sr-stage-practice);
    }
    .topic {
        margin: 11px 0 0;
        font-size: 20px;
        font-weight: 700;
        line-height: 1.2;
        color: var(--sr-ink);
    }

    .transition {
        display: inline-flex;
        flex-wrap: wrap;
        align-items: center;
        justify-content: center;
        gap: 9px;
        margin: 8px 0 0;
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 600;
        letter-spacing: 0.06em;
    }
    .transition .from {
        color: var(--sr-ink-3);
        text-transform: uppercase;
    }
    .transition .arrow {
        color: var(--sr-stage-practice);
    }
    .transition .to {
        color: var(--sr-stage-practice-deep);
        text-transform: uppercase;
    }
    .transition .count {
        font-weight: 500;
        color: var(--sr-faint);
    }

    .rows {
        list-style: none;
        margin: 20px 0 0;
        padding: 0;
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        justify-content: center;
    }
    .row {
        display: flex;
        align-items: center;
        gap: 8px;
        box-sizing: border-box;
        width: 250px;
        max-width: 100%;
        padding: 9px 12px;
        background: var(--sr-white);
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-control);
        text-align: left;
    }
    .badge {
        flex-shrink: 0;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 17px;
        height: 17px;
        border-radius: 50%;
        background: var(--sr-stage-practice-soft);
        color: var(--sr-stage-practice-deep);
        font-family: var(--sr-mono);
        font-size: 10px;
        font-weight: 700;
        line-height: 1;
    }
    .name {
        flex: 1;
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        font-size: 11.5px;
        font-weight: 600;
        color: var(--sr-ink);
    }
    .glyph {
        flex-shrink: 0;
        display: inline-flex;
    }
    // Every glyph pops as it flips to the reached stage, together.
    .rows.advanced .glyph {
        animation: sr-unison 420ms cubic-bezier(0.2, 0.9, 0.3, 1.2) both;
    }
    @keyframes sr-unison {
        0% {
            transform: scale(0.7);
            opacity: 0.4;
        }
        60% {
            transform: scale(1.12);
        }
        100% {
            transform: scale(1);
            opacity: 1;
        }
    }

    // Full-card, transparent advance target; the card shows no explicit button.
    .overlay {
        position: absolute;
        inset: 0;
        z-index: 2;
        margin: 0;
        padding: 0;
        border: 0;
        background: transparent;
        cursor: pointer;
        border-radius: var(--sr-radius-card);
    }
    .overlay:focus-visible {
        outline: 2px solid var(--sr-signal);
        outline-offset: -3px;
    }

    // Reduced motion: keep the earned after-state, drop every moving part.
    .card.reduced .glyph,
    .card.reduced .confetti {
        animation: none;
    }
    @media (prefers-reduced-motion: reduce) {
        .confetti {
            display: none;
        }
        .rows.advanced .glyph {
            animation: none;
        }
    }
</style>
