<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The grading step that rides at the bottom of graded cards (Practice, Guided,
Solo — never Learn). "How hard was this card?" then Again / Hard / Good / Easy
grades the card through FSRS; the picked grade stays outlined and the resulting
next interval shows inline, with "Next card" to continue. The backend call is
passed in so the interval it returns is shown here on the same card.
-->
<script lang="ts">
    import { type AnswerResult, type Rating, RATINGS } from "./lib";

    // Grades the card through FSRS and resolves with the answer result (which
    // carries the next interval). A prop so the orchestrator owns the RPC.
    export let answer: (rating: Rating) => Promise<AnswerResult>;
    // Advance the session, carrying the result (a level-up may follow).
    export let onDone: (result: AnswerResult) => void;
    // Surface a grading failure to the orchestrator's error view.
    export let onError: (err: unknown) => void = () => {};

    // Ordered Again/Hard/Good/Easy -> the tint + text token key for each.
    const TONE = ["again", "hard", "good", "easy"] as const;

    type Phase = "rate" | "grading" | "done";
    let phase: Phase = "rate";
    let picked: Rating | null = null;
    let result: AnswerResult | null = null;

    async function rate(rating: Rating): Promise<void> {
        if (phase !== "rate") {
            return;
        }
        picked = rating;
        phase = "grading";
        try {
            result = await answer(rating);
            phase = "done";
        } catch (err) {
            phase = "rate";
            picked = null;
            onError(err);
        }
    }

    function proceed(): void {
        if (result) {
            onDone(result);
        }
    }

    // 1..4 grade the card (base Anki's ease shortcuts); Enter continues once the
    // interval is shown. Typing in a field is never eaten.
    function onKey(event: KeyboardEvent): void {
        if (event.metaKey || event.ctrlKey || event.altKey) {
            return;
        }
        const target = event.target as HTMLElement | null;
        if (target && (target.tagName === "TEXTAREA" || target.tagName === "INPUT")) {
            return;
        }
        if (phase === "rate") {
            const n = Number(event.key);
            if (n >= 1 && n <= 4) {
                event.preventDefault();
                rate(n as Rating);
            }
        } else if (phase === "done" && (event.key === "Enter" || event.key === " ")) {
            event.preventDefault();
            proceed();
        }
    }
</script>

<svelte:window on:keydown={onKey} />

<p class="ask">How hard was this card?</p>
<div class="row" role="group" aria-label="Rate difficulty">
    {#each RATINGS as choice (choice.rating)}
        <button
            class="grade grade--{TONE[choice.rating - 1]}"
            class:selected={picked === choice.rating}
            type="button"
            on:click={() => rate(choice.rating)}
            disabled={phase !== "rate"}
            aria-pressed={picked === choice.rating}
        >
            {choice.label}
        </button>
    {/each}
</div>

{#if phase === "done" && result && picked}
    <p class="result">
        <span class="verdict verdict--{TONE[picked - 1]}">
            {RATINGS[picked - 1].label.toUpperCase()}
        </span>
        {#if result.intervalText}
            <span class="next">· next review in {result.intervalText}</span>
        {:else}
            <span class="next">· recorded</span>
        {/if}
        <button class="proceed" type="button" on:click={proceed}>Next card →</button>
    </p>
{/if}

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .ask {
        margin: 0 0 10px;
        font-size: 12.5px;
        font-weight: 700;
        color: var(--sr-ink);
    }
    .row {
        display: flex;
        gap: 6px;
    }
    .grade {
        @include syn.grade;
    }
    .grade:disabled {
        cursor: default;
    }
    // Not-picked grades dim once a grade is locked in, so the choice reads.
    .row:has(.selected) .grade:not(.selected) {
        opacity: 0.5;
    }
    .grade--again {
        background: var(--sr-signal-weak);
        color: var(--sr-signal-deep);
    }
    .grade--hard {
        @include syn.stage-tint(guided);
    }
    .grade--good {
        @include syn.stage-tint(practice);
    }
    .grade--easy {
        @include syn.stage-tint(solo);
    }
    .grade--again.selected {
        outline: 2px solid var(--sr-signal);
        outline-offset: -2px;
    }
    .grade--hard.selected {
        outline: 2px solid var(--sr-stage-guided);
        outline-offset: -2px;
    }
    .grade--good.selected {
        outline: 2px solid var(--sr-stage-practice);
        outline-offset: -2px;
    }
    .grade--easy.selected {
        outline: 2px solid var(--sr-stage-solo);
        outline-offset: -2px;
    }

    .result {
        display: flex;
        align-items: center;
        gap: 7px;
        margin: 11px 0 0;
        font-family: var(--sr-mono);
        font-size: 9px;
        animation: sr-result-in 240ms ease both;
    }
    @keyframes sr-result-in {
        from {
            opacity: 0;
            transform: translateY(3px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    .verdict {
        font-weight: 600;
        letter-spacing: 0.06em;
    }
    .verdict--again {
        color: var(--sr-signal-deep);
    }
    .verdict--hard {
        color: var(--sr-stage-guided-deep);
    }
    .verdict--good {
        color: var(--sr-stage-practice-deep);
    }
    .verdict--easy {
        color: var(--sr-stage-solo-deep);
    }
    .next {
        font-weight: 500;
        color: var(--sr-faint);
    }
    .proceed {
        margin-left: auto;
        padding: 0;
        border: none;
        background: none;
        font-family: var(--sr-mono);
        font-size: 9px;
        font-weight: 600;
        color: var(--sr-ink);
        cursor: pointer;
    }
    .proceed:hover {
        color: var(--sr-signal-ink);
    }

    @media (prefers-reduced-motion: reduce) {
        .result {
            animation: none;
        }
    }
</style>
