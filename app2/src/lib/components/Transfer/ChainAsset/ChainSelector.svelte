<script lang="ts">
import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte.ts"
import { cn } from "$lib/utils"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { chainLogoMap } from "$lib/constants/chain-logos.ts"

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
            "flex flex-col items-center gap-2 justify-start px-2 py-4 rounded-md transition-colors cursor-pointer",
            (type === "source" && transfer.raw.source === chain.chain_id) ||
            (type === "destination" && transfer.raw.destination === chain.chain_id)
              ? "bg-zinc-900 hover:bg-zinc-800 ring-1 ring-sky-500"
              : "bg-zinc-900 hover:bg-zinc-800"
          )}
                onclick={() => selectChain(chain)}
        >

          {#if chain.universal_chain_id}
            {@const chainLogo = chainLogoMap.get(chain.universal_chain_id)}
            {#if chainLogo?.color}
              <span class="w-10 h-10 flex items-center justify-center overflow-hidden">
                <img src={chainLogo.color} alt="">
              </span>
            {/if}
          {/if}

          <span class="text-xs text-center truncate w-fit">{chain.display_name}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="py-2 text-center text-zinc-500">
      <span class="inline-block animate-pulse">Loading chains...</span>
    </div>
  {/if}
</div>