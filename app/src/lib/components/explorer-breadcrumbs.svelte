<script lang="ts">
import { page } from "$app/stores"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import TablePackets from "$lib/components/tables/packets.svelte"
import { derived } from "svelte/store"

const pathComponents = derived(page, $page => $page.url.pathname.split("/").slice(2))
</script>

<nav class="flex">
{#each $pathComponents as pathComponent, index} 
  {@const pathUrl = $page.url.pathname.split("/").slice(0, index+3).join("/")}
  <a class="block border-b px-2 py-1 border-r" href={pathUrl}>{pathComponent}</a>
{/each}
</nav>
