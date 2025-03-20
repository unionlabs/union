<script lang="ts">
import { cn } from "$lib/utils/index.js"
import { Option } from "effect"
import { transfer } from "$lib/components/Transfer/transfer.svelte.js"
import Label from "$lib/components/ui/Label.svelte"

type Props = {
  type: "source" | "destination"
  onClick: () => void
}

const { type, onClick }: Props = $props()

const selectedChain = $derived.by(() => {
  if (type === "source") {
    return transfer.sourceChain
  }
  return transfer.destinationChain
})

const isChainLoading = $derived.by(() => {
  if (type === "source") {
    return transfer.raw.source && Option.isNone(transfer.sourceChain)
  }
  return transfer.raw.destination && Option.isNone(transfer.destinationChain)
})
</script>

<div>
  <Label>{type}</Label>
  <button
          onclick={onClick}
          class={cn(
      "w-full rounded-lg border border-zinc-600 bg-zinc-700 text-zinc-200",
      "hover:bg-zinc-600 hover:border-zinc-500",
      "focus:outline-none focus:ring-2 focus:ring-sky-500",
      "disabled:opacity-50 disabled:cursor-not-allowed",
      "transition-all duration-200 cursor-pointer",
      "p-0 overflow-hidden"
    )}
  >
    <div class="flex items-center">
      {#if isChainLoading}
        <!-- Loading State -->
        <div class="flex items-center justify-center p-2 flex-1">
          <span>Loading...</span>
        </div>
      {:else if Option.isNone(selectedChain)}
        <!-- No Chain Selected -->
        <div class="flex items-center justify-between p-2 flex-1">
          <span class="text-zinc-400">Select {type}</span>
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
               stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
               class="text-zinc-400">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </div>
      {:else}
        <!-- Chain Selected -->
        <div class="flex items-center bg-zinc-800 p-2 border-r border-zinc-600">
          <div class="w-8 h-8 rounded-full bg-sky-500 flex items-center justify-center overflow-hidden">
            <span class="text-xs font-medium uppercase">{selectedChain.value.display_name.substring(0, 2)}</span>
          </div>
        </div>

        <!-- Asset part (for both source and destination) -->
        <div class="flex-1 p-2 flex items-center justify-between">
          {#if type === "source" && transfer.raw.asset && Option.isNone(transfer.baseToken)}
            <!-- Asset Loading (only for source) -->
            <span class="flex items-center">
              <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none"
                   viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <span>Loading...</span>
            </span>
          {:else if Option.isSome(transfer.baseToken)}
            <!-- Selected Asset (both source and destination) -->
            <div class="flex items-center">
              <!-- Show the asset, grayed out for destination type -->
              <span class={type === "destination" ? "truncate text-zinc-400" : "truncate"}>
                {transfer.baseToken.value.representations[0]?.name ?? transfer.baseToken.value.denom}
              </span>
            </div>
          {:else if type === "source"}
            <!-- No Asset Selected (only for source) -->
            <span class="text-zinc-400">Select asset</span>
          {:else}
            <!-- No Asset Selected (for destination) -->
            <span class="text-zinc-400">No asset selected</span>
          {/if}
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
               stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
               class="text-zinc-400">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </div>
      {/if}
    </div>
  </button>
</div>