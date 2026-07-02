<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

Shared card chrome for the study interactions. The footer slot sits at the
bottom center, where the difficulty rating (or a continue action) lives.
-->
<div class="card">
    <div class="top">
        <slot name="header" />
    </div>
    <div class="content">
        <slot />
    </div>
    <div class="foot">
        <slot name="footer" />
    </div>
</div>

<style lang="scss">
    .card {
        width: 100%;
        max-width: 46rem;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        background: var(--sr-panel);
        border: 1px solid var(--sr-line);
        border-radius: 12px;
        padding: 1.5rem 1.6rem 1.5rem;
    }
    .top {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        min-height: 1.2rem;
    }
    .content {
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
    }
    // Bottom center: the ease buttons or a continue action stand here, in the
    // spot base Anki reserves for its detached bottom bar.
    .foot {
        display: flex;
        justify-content: center;
        min-height: 2.6rem;
        align-items: center;
    }
    @media (max-width: 34rem) {
        .card {
            padding: 1.1rem 1rem 1.2rem;
            border-radius: 10px;
        }
    }
</style>
