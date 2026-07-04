<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

A Speedrun figure: a collection-media image referenced by bare filename and
served root-relative (`/<filename>`) by the desktop mediasrv and the AnkiDroid
shell's webview client. Renders nothing when the filename is empty/undefined, so
callers can pass an optional field straight through.
-->
<script lang="ts">
    export let filename: string | null | undefined = undefined;
    export let alt = "";
    // Drop the leading margin when the image sits inside another element (e.g. a
    // choice button) that already owns the spacing.
    export let compact = false;

    $: src = (filename ?? "").trim();
</script>

{#if src}
    <img class="sr-media" class:compact src="/{src}" {alt} loading="lazy" />
{/if}

<style lang="scss">
    .sr-media {
        display: block;
        max-width: 100%;
        height: auto;
        margin: 10px 0 0;
        border: 1px solid var(--sr-line);
        border-radius: var(--sr-radius-tile);
    }
    .compact {
        margin: 0;
    }
</style>
