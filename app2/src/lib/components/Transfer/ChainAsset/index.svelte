<script lang="ts">
import { Option } from "effect"
import { fade, fly } from "svelte/transition"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import ChainAssetButton from "$lib/components/Transfer/ChainAsset/ChainAssetButton.svelte"
import ChainSelector from "$lib/components/Transfer/ChainAsset/ChainSelector.svelte"
import TransferDirectionInfo from "$lib/components/Transfer/ChainAsset/TransferDirectionInfo.svelte"
import AssetSelector from "$lib/components/Transfer/ChainAsset/AssetSelector.svelte"

type Props = {
  type: "source" | "destination"
}

const { type }: Props = $props()
let open = $state(false)

function ensureTokensForChain() {
  if (type !== "source" || Option.isNone(transfer.sourceChain)) return

  const chainId = transfer.sourceChain.value.universal_chain_id
  if (!chainId) return

  const tokenData = tokensStore.getData(chainId)
  if (Option.isNone(tokenData)) {
    tokensStore.fetchTokens(chainId)
  }
}

$effect(() => {
  if (type === "source" && Option.isSome(transfer.sourceChain)) {
    ensureTokensForChain()
  }
})

function closeModal() {
  open = false
}

function onChainSelected() {
  if (type === "destination") {
    closeModal()
  }
}

function onAssetSelected() {
  closeModal()
}
</script>

{#if open}
  <div class="absolute inset-0 bg-zinc-900 z-40 flex" transition:fade={{ duration: 300 }}>
    <div class="w-full h-auto max-h-full flex flex-col overflow-hidden" transition:fly={{ y: 30, duration: 300, opacity: 0 }}>
      <!-- Header with close button -->
      <div class="p-4 border-b border-zinc-700 flex justify-between items-center">
        <h3 class="text-sm font-medium text-zinc-400">Select {type.charAt(0).toUpperCase() + type.slice(1)}</h3>
        <button
                class="text-zinc-400 hover:text-white"
                onclick={closeModal}
        >
          <span class="sr-only">close</span>
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- Modal Content with proper scrolling -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <!-- Chains Section -->
        <ChainSelector {type} onSelect={onChainSelected} />

        <!-- Assets Section - Only show for source type -->
        {#if type === "source"}
          <AssetSelector onSelect={onAssetSelected} />
          {:else}
          <TransferDirectionInfo />
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- Chain Asset Button -->
<ChainAssetButton {type} onClick={() => open = true} />