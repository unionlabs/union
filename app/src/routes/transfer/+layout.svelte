<script lang="ts">
import { onMount } from "svelte"
import { page } from "$app/stores"

/**
 * Clear all search params from the URL when navigating away from /transfer
 * aka when component is unmounted
 */
onMount(() => {
  return () => {
    $page.url.searchParams.forEach(searchParam => $page.url.searchParams.delete(searchParam))
  }
})
</script>

<slot />
