<script lang="ts">
import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte.ts"
import { cn } from "$lib/utils"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import type { Chain } from "$lib/schema/chain.ts"

type Props = {
  type: "source" | "destination"
  onSelect: () => void
}

const { type, onSelect }: Props = $props()

function selectChain(chain: Chain) {
  transfer.raw.updateField(type, chain.chain_id)
  if (type === "source") {
    tokensStore.fetchTokens(chain.universal_chain_id)
  }
  onSelect()
}
</script>

<div class="p-4">
  {#if Option.isSome(chains.data)}
    <div class="grid grid-cols-3 gap-2">
      {#each chains.data.value as chain}
        <button
                class={cn(
            "flex items-center gap-1 justify-start p-2 rounded-md transition-colors",
            "border border-zinc-700 hover:bg-zinc-700",
            (type === "source" && transfer.raw.source === chain.chain_id) ||
            (type === "destination" && transfer.raw.destination === chain.chain_id)
              ? "bg-zinc-700 ring-2 ring-sky-500"
              : "bg-zinc-800/50"
          )}
                onclick={() => selectChain(chain)}
        >
          <span class="w-4 h-4 rounded-full bg-sky-500 flex items-center justify-center overflow-hidden">
            <span class="text-xs font-medium uppercase">{chain.display_name.substring(0, 1)}</span>
          </span>
          <span class="text-xs text-center truncate w-fit">{chain.display_name.split(" ")[0]}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="py-2 text-center text-zinc-500">
      <span class="inline-block animate-pulse">Loading chains...</span>
    </div>
  {/if}
</div>