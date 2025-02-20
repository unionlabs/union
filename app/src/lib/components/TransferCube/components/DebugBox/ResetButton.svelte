<script lang="ts">
import { Button } from "$lib/components/ui/button"

export let onReset: () => void
let reset = false

async function handleClick() {
  try {
    await onReset()
    reset = true
    setTimeout(() => {
      reset = false
    }, 2000)
  } catch (err) {
    console.error("Failed to reset:", err)
  }
}
</script>

<Button
        variant="outline"
        on:click={handleClick}
        class="flex items-center gap-2"
>
  {#key reset}
    <div class="flex items-center gap-2">
      {#if reset}
        <svg xmlns="http://www.w3.org/2000/svg" class="size-5" viewBox="0 0 24 24">
          <path fill="currentColor" d="m9.55 18l-5.7-5.7l1.425-1.425L9.55 15.15l9.175-9.175L20.15 7.4z"/>
        </svg>
        <span>Reset!</span>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" class="size-5" viewBox="0 0 24 24">
          <path fill="currentColor" d="M22 12c0 5.523-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2v2a8 8 0 1 0 5.135 1.865L15 8V2h6l-2.447 2.447A9.98 9.98 0 0 1 22 12"/>
        </svg>
        <span>Reset</span>
      {/if}
    </div>
  {/key}
</Button>