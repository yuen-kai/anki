<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { onMount } from "svelte";

    import { goto } from "$app/navigation";

    import { startAutoSync } from "../speedrun-account/autosync.svelte";
    import { deleteDeck, isMobileShell, openDeck } from "../speedrun-hierarchy/lib";
    import DecksView from "./DecksView.svelte";
    import type { DeckRow } from "./lib";
    import type { PageData } from "./$types";

    export let data: PageData;

    // The Speedrun shell boots here on both platforms, so start periodic
    // background auto-sync once for the app's lifetime (idempotent).
    onMount(startAutoSync);

    // Account (profile avatar) and import (New deck) are in-app on every
    // platform; only Demo still lives in the desktop Tools menu, so the mobile
    // shell surfaces it here.
    const mobile = isMobileShell();

    function create(): void {
        goto("/speedrun-import");
    }

    function details(deck: DeckRow): void {
        goto(`/speedrun-hierarchy/${deck.deckId}`);
    }
</script>

<DecksView
    decks={data.decks}
    loadError={data.error}
    {mobile}
    onStudy={(deck) => openDeck(deck.deckId)}
    onDetails={details}
    onCreate={create}
    onDelete={(deck) => deleteDeck(deck.deckId)}
    onAccount={() => goto("/speedrun-account")}
    onDemo={() => goto("/speedrun-review-demo")}
/>
