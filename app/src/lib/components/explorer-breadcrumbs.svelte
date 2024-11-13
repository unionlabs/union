<script lang="ts">
import { page } from "$app/stores"
import { onMount } from 'svelte'
// import { derived } from "svelte/store"

// const pathComponents = derived(page, $page => $page.url.pathname.split("/").slice(2))

const pathComponents = $derived($page.url.pathname.split("/").slice(2))

onMount(() => {
  console.info(pathComponents)
})
</script>

<nav>
  <div class="flex overflow-x-auto text-sm">
    {#each pathComponents as pathComponent, index} 
      {@const pathUrl = $page.url.pathname.split("/").slice(0, index+3).join("/")}
      <a class="block border-b px-2 py-1 border-r" href={pathUrl}>{pathComponent}</a>
    {/each}
  </div>
</nav>
