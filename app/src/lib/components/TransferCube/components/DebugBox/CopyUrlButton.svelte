<script lang="ts">
import { get } from "svelte/store"
import { page } from "$app/stores"
import { Button } from "$lib/components/ui/button"

let copied = false

async function copyUrl() {
  try {
    await navigator.clipboard.writeText(get(page).url.href)
    copied = true
    setTimeout(() => {
      copied = false
    }, 2000)
  } catch (err) {
    console.error("Failed to copy URL:", err)
  }
}
</script>

<Button
        variant="outline"
        on:click={copyUrl}
        class="flex items-center gap-2"
>
  {#key copied}
    <div class="flex items-center gap-2">
      {#if copied}
        <svg xmlns="http://www.w3.org/2000/svg" class="size-5" viewBox="0 0 24 24">
          <path fill="currentColor" d="m9.55 18l-5.7-5.7l1.425-1.425L9.55 15.15l9.175-9.175L20.15 7.4z"/>
        </svg>
        <span>Copied!</span>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" class="size-5" viewBox="0 0 24 24">
          <path fill="currentColor"
                d="M9 18q-.825 0-1.412-.587T7 16V4q0-.825.588-1.412T9 2h9q.825 0 1.413.588T20 4v12q0 .825-.587 1.413T18 18zm0-2h9V4H9zm-4 6q-.825 0-1.412-.587T3 20V6h2v14h11v2zm4-6V4z"/>
        </svg>
        <span>Share URL</span>
      {/if}
    </div>
  {/key}
</Button>