<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

The new-user welcome: a centred card that greets a first-run learner, names the
four learning levels and how concepts auto-upgrade, and offers a single Start
studying CTA. Purely presentational; the host wires the callback.
-->
<script lang="ts">
    export let onStart: () => void;
</script>

<section class="card">
    <div class="glow" aria-hidden="true"></div>
    <div class="body">
        <p class="eyebrow">Getting started</p>
        <h1 class="title">Welcome to Synapse</h1>
        <p class="copy">
            Concepts have 4 learning levels and automatically upgrade as you progress,
            taking you from learning to answering MCAT questions. Synapse uses learning
            science to organize and pace your learning. Simply click start studying and
            we will handle the rest.
        </p>
        <button class="start" type="button" on:click={onStart}>Start studying</button>
    </div>
</section>

<style lang="scss">
    @use "$lib/sass/speedrun-synapse" as syn;

    .card {
        position: relative;
        overflow: hidden;
        box-sizing: border-box;
        width: 100%;
        max-width: 42rem;
        margin: 0 auto;
        padding: 32px 28px 34px;
        font-family: var(--sr-sans);
        @include syn.card;
    }
    .glow {
        position: absolute;
        inset: 0;
        pointer-events: none;
        background: radial-gradient(
            circle at 50% -10%,
            color-mix(in srgb, var(--sr-signal) 10%, transparent),
            transparent 45%
        );
    }
    .body {
        position: relative;
        text-align: center;
    }

    .eyebrow {
        @include syn.eyebrow;
        margin: 0;
        color: var(--sr-signal);
    }
    .title {
        margin: 12px 0 0;
        font-size: 22px;
        font-weight: 700;
        letter-spacing: var(--sr-tighten);
        color: var(--sr-ink);
    }
    .copy {
        max-width: 42ch;
        margin: 12px auto 0;
        font-size: 14px;
        line-height: 1.6;
        color: var(--sr-ink-2);
    }

    .start {
        @include syn.btn;
        @include syn.btn-primary;
        @include syn.btn-hero;
        margin-top: 22px;
        box-shadow: var(--sr-shadow-signal);
    }
    .start:hover {
        filter: brightness(1.04);
    }

    @media (max-width: 40rem) {
        .card {
            padding: 26px 20px 28px;
            border-radius: var(--sr-radius-card-sm);
            box-shadow: var(--sr-shadow-phone);
        }
        .start {
            min-height: 44px;
        }
    }
</style>
