<script lang="ts">
import { Option } from "effect"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"

// This component shows a visual representation of the transfer direction
</script>

<div class="flex-1 flex flex-col items-center justify-center p-6 text-center">
  <div class="flex items-center space-x-4 mb-4">
    <!-- Source info -->
    {#if Option.isSome(transfer.sourceChain)}
      <div class="flex flex-col items-center">
        <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center mb-2 border border-zinc-700">
          <span class="text-sm font-medium uppercase">{transfer.sourceChain.value.display_name.substring(0, 2)}</span>
        </div>
        <span class="text-sm text-zinc-300">{transfer.sourceChain.value.display_name.split(" ")[0]}</span>
      </div>
    {:else}
      <div class="flex flex-col items-center">
        <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center mb-2 border border-zinc-700">
          <span class="text-sm">?</span>
        </div>
      </div>
    {/if}

    <!-- Arrow -->
    <div class="flex flex-col items-center">
      <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-sky-400">
        <path d="M5 12h14"></path>
        <path d="m12 5 7 7-7 7"></path>
      </svg>
    </div>

    <!-- Destination info or placeholder -->
    {#if Option.isSome(transfer.destinationChain)}
      <div class="flex flex-col items-center">
        <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center mb-2 border border-zinc-700">
          <span class="text-sm font-medium uppercase">{transfer.destinationChain.value.display_name.substring(0, 2)}</span>
        </div>
        <span class="text-sm text-zinc-300">{transfer.destinationChain.value.display_name.split(" ")[0]}</span>
      </div>
    {:else}
      <div class="flex flex-col items-center">
        <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center mb-2 border border-zinc-700 border-dashed animate-pulse">
          <span class="text-sm">?</span>
        </div>

      </div>
    {/if}
  </div>

  {#if Option.isNone(transfer.destinationChain)}
    <p class="text-zinc-400 text-sm max-w-xs">
      Please select a destination chain to continue with your transfer
    </p>
  {/if}
</div>