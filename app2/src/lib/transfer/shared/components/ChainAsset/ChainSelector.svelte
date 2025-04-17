<script lang="ts">
import { Array as Arr, flow, Option } from "effect"
import { chains } from "$lib/stores/chains.svelte.ts"
import { cn } from "$lib/utils"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { chainLogoMap } from "$lib/constants/chain-logos.ts"
import { MODE } from "$lib/constants/config"

type Props = {
  type: "source" | "destination"
  onSelect: () => void
}

const { type, onSelect }: Props = $props()

function selectChain(chain: Chain) {
  if (type === "destination" && chain.chain_id === transferData.raw.source) {
    return // Don't allow selecting same chain as destination
  }

  transferData.raw.updateField(type, chain.chain_id)
  if (type === "source") {
    tokensStore.fetchTokens(chain.universal_chain_id)
    if (transferData.raw.destination === chain.chain_id) {
      transferData.raw.updateField("destination", "")
    }
  }

  onSelect()
}

const testnets = ["ethereum.11155111", "corn.21000001", "bob.808813", "babylon.bbn-test-5"]

const mainnets = ["bob.60808", "corn.21000000", "babylon.bbn-1", "ethereum.1"]

// For btc.union.build, only show bitcoin chains
const filteredChains = $derived(
  chains.data.pipe(
    Option.map(
      flow(
        Arr.filter(c => {
          const allowed = MODE === "testnet" ? testnets : mainnets
          return allowed.includes(c.universal_chain_id)
        })
      )
    )
  )
)
</script>

<div class="p-4">
  {#if Option.isSome(filteredChains)}
    {@const chainss = filteredChains.value}
    <div class="grid grid-cols-3 gap-2">
      {#each chainss as chain}
        {@const isSelected =
          (type === "source" && transferData.raw.source === chain.chain_id) ||
          (type === "destination" &&
            transferData.raw.destination === chain.chain_id)}
        {@const isDisabled =
          type === "destination" && transferData.raw.source === chain.chain_id}

        <button
          class={cn(
            "flex flex-col items-center gap-2 justify-start px-2 py-4 rounded-md transition-colors",
            isSelected
              ? "bg-zinc-900 hover:bg-zinc-800 ring-1 ring-babylon-orange"
              : isDisabled
                ? "bg-zinc-900 opacity-50 cursor-not-allowed"
                : "bg-zinc-900 hover:bg-zinc-800 cursor-pointer",
          )}
          onclick={() => !isDisabled && selectChain(chain)}
          disabled={isDisabled}
        >
          {#if chain.universal_chain_id}
            {@const chainLogo = chainLogoMap.get(chain.universal_chain_id)}
            {#if chainLogo?.color}
              <span
                class="w-10 h-10 flex items-center justify-center overflow-hidden"
              >
                <img src={chainLogo.color} alt="" />
              </span>
            {/if}
          {/if}

          <span class="text-xs text-center truncate w-fit"
            >{chain.display_name}</span
          >

          {#if isDisabled}
            <span class="text-xs text-sky-400 -mt-2">From Chain</span>
          {/if}
        </button>
      {/each}
    </div>
  {:else}
    <div class="py-2 text-center text-zinc-500">
      <span class="inline-block animate-pulse">Loading chains...</span>
    </div>
  {/if}
</div>
