<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto } from "$app/navigation";

    import HierarchyEditor from "../HierarchyEditor.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;
</script>

{#if data.hierarchy}
    <HierarchyEditor hierarchy={data.hierarchy} />
{:else}
    <div class="load-error">
        <p>Could not open this deck: {data.error}</p>
        <button on:click={() => goto("/speedrun-decks")}>Back to decks</button>
    </div>
{/if}

<style lang="scss">
    @use "$lib/sass/speedrun-tokens" as sr;

    .load-error {
        @include sr.tokens;

        box-sizing: border-box;
        min-height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 1rem;
        padding: 4rem 1.5rem;
        background: var(--sr-paper);
        color: var(--sr-ink-2);
        font-family: var(--sr-sans);
        text-align: center;
    }
    .load-error p {
        margin: 0;
    }
    .load-error button {
        border: 1px solid var(--sr-line-2);
        background: var(--sr-panel);
        color: var(--sr-ink);
        border-radius: 6px;
        padding: 0.5rem 0.9rem;
        font: inherit;
        font-weight: 560;
        cursor: pointer;
    }
    .load-error button:hover {
        border-color: var(--sr-ink-3);
    }
    .load-error :global(:focus-visible) {
        outline: 2px solid var(--sr-signal);
        outline-offset: 2px;
        border-radius: 4px;
    }
</style>
