<script lang="ts">
import { cn } from "$lib/utils/index.js"
import { Option } from "effect"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import Label from "$lib/components/ui/Label.svelte"
import { chainLogoMap } from "$lib/constants/chain-logos.ts"
import SharpChevronDownIcon from "$lib/components/icons/SharpChevronDownIcon.svelte"
import LoadingSpinnerIcon from "$lib/components/icons/LoadingSpinnerIcon.svelte"

type Props = {
  type: "source" | "destination"
  onClick: () => void
}

const { type, onClick }: Props = $props()

const selectedChain = $derived.by(() => {
  if (type === "source") {
    return transferData.sourceChain
  }
  return transferData.destinationChain
})

const isChainLoading = $derived.by(() => {
  if (type === "source") {
    return transferData.raw.source && Option.isNone(transferData.sourceChain)
  }
  return transferData.raw.destination && Option.isNone(transferData.destinationChain)
})
</script>

<div class="w-full">
  <Label class="pb-1">{type === "source" ? "From" : "To"}</Label>
  <button
          onclick={onClick}
          class={cn(
      "w-full h-14 rounded-md bg-zinc-800/70 text-zinc-200",
      "hover:bg-zinc-800 hover:border-zinc-500",
      "focus:outline-none focus:ring-1 focus:ring-babylon-orange",
      "disabled:opacity-50 disabled:cursor-not-allowed",
      "transition-all duration-200 cursor-pointer",
    )}
  >
    <div class="flex items-center">
      {#if isChainLoading}
        <div class="flex gap-2 items-center justify-between p-2 flex-1">
          <div class="w-8 h-8 flex items-center bg-zinc-500 text-white rounded-full justify-center">
            <LoadingSpinnerIcon/>
          </div>
          <span class="text-zinc-400">Loading...</span>
          <div class="text-transparent">
            <SharpChevronDownIcon/>
          </div>
        </div>
      {:else if Option.isNone(selectedChain)}
        <!-- No Chain Selected -->
        <div class="flex gap-2 items-center justify-between p-3 flex-1">
          <div class="w-8 h-8 flex items-center bg-zinc-700 rounded-full justify-center"></div>
          <span class="text-zinc-400 flex-1 text-start">Select</span>
          <SharpChevronDownIcon class="size-6"/>
        </div>
      {:else}

        <!-- Chain Selected -->
        <div class="flex gap-2 items-center justify-between p-3 flex-1 w-full">

          <!--LOGO-->
          {#if selectedChain.value.universal_chain_id}
            {@const chainLogo = chainLogoMap.get(selectedChain.value.universal_chain_id)}
            {#if chainLogo?.color}
              <div class="flex items-center">
                <div class="size-8 flex items-center justify-center overflow-hidden">
                  <img src={chainLogo.color} alt="">
                </div>
              </div>
            {/if}
          {/if}

          {#if type === "source" && transferData.raw.asset && Option.isNone(transferData.baseToken)}
            <!-- Asset Loading (only for source) -->
            <span class="flex items-center">
              <span>Loading...</span>
            </span>
          {:else if Option.isSome(transferData.baseToken)}
            <!-- Selected Asset (both source and destination) -->
            <!-- Show the asset, grayed out for destination type -->
            <div class={cn(
              type === "destination" ? "truncate" : "truncate",
              "flex flex-col items-start w-full"
              )}>
              <p class="leading-4 font-bold">
                {transferData.baseToken.value.representations[0]?.symbol ?? transferData.baseToken.value.denom}
              </p>
              {#if Option.isSome(transferData.sourceChain)}
                <p class="text-xs text-zinc-400">{ type === "source" ? transferData.sourceChain.value.display_name : transferData.destinationChain.value.display_name }</p>
              {/if}

            </div>
          {:else if type === "source"}
            <span class="text-zinc-400 flex-1 text-start">Select</span>
          {:else}
            <span class="text-zinc-400 flex-1 text-start">No asset</span>
          {/if}
          <SharpChevronDownIcon class="size-6"/>
        </div>
      {/if}
    </div>
  </button>
</div>
