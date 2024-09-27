<script lang="ts">
import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { cn } from "$lib/utils/utils.ts"

const { terminal } = getState()
</script>

<div class="flex flex-col">
  {#each terminal.history as content}
    {#if content.lineBreak}
      <Print><br></Print>
    {:else}
      <p class={cn(
        "w-fit text-neutral-300 text-sm sm:text-base",
        content.type === "warning" ? "!text-[#FD6363]" : "",
        content.type === "info" ? "!text-union-accent-500" : "",
        content.uppercase ? "uppercase" : "",
        )}>{@html content.text}</p>
    {/if}
  {/each}
</div>