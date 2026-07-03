<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { goto } from "$app/navigation";

    import { deleteDeck, openDeck } from "../speedrun-hierarchy/lib";
    import DecksView from "./DecksView.svelte";
    import type { DeckRow } from "./lib";
    import type { PageData } from "./$types";

    export let data: PageData;

    function create(): void {
        goto("/speedrun-hierarchy/new");
    }

    function details(deck: DeckRow): void {
        goto(`/speedrun-hierarchy/${deck.deckId}`);
    }
</script>

<DecksView
    decks={data.decks}
    loadError={data.error}
    onStudy={(deck) => openDeck(deck.deckId)}
    onDetails={details}
    onCreate={create}
    onDelete={(deck) => deleteDeck(deck.deckId)}
/>
