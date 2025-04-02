<script lang="ts">
import { fade, fly } from "svelte/transition"
import { onDestroy, onMount } from "svelte"
import ChainAssetButton from "$lib/components/Transfer/ChainAsset/ChainAssetButton.svelte"
import ChainSelector from "$lib/components/Transfer/ChainAsset/ChainSelector.svelte"
import TransferDirectionInfo from "$lib/components/Transfer/ChainAsset/TransferDirectionInfo.svelte"
import AssetSelector from "$lib/components/Transfer/ChainAsset/AssetSelector.svelte"
import SharpCancelIcon from "$lib/components/icons/SharpCancelIcon.svelte"

type Props = {
  type: "source" | "destination"
}

const { type }: Props = $props()
let open = $state(false)

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

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && open) {
    closeModal()
  }
}

onMount(() => {
  document.addEventListener("keydown", handleKeydown)
})

onDestroy(() => {
  document.removeEventListener("keydown", handleKeydown)
})
</script>

{#if open}
  <div class="absolute inset-0 bg-zinc-900 z-40 flex" transition:fade={{ duration: 300 }}>
    <div class="w-full h-auto max-h-full flex flex-col overflow-hidden" transition:fly={{ y: 30, duration: 300, opacity: 0 }}>
      <!-- Header with close button -->
      <div class="p-4 border-b border-zinc-800 flex justify-between items-center h-12">
        <h3 class="text-sm font-medium text-zinc-400">Select {type.charAt(0).toUpperCase() + type.slice(1)}</h3>
        <button
                aria-label="Close"
                class="text-zinc-400 hover:text-white cursor-pointer"
                onclick={closeModal}
        >
          <SharpCancelIcon />
        </button>
      </div>

      <!-- Modal Content with proper scrolling -->
      <div class="flex flex-col h-full">
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