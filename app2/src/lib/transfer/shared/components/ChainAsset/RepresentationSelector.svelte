<script lang="ts">
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import TransferAsset from "$lib/transfer/shared/components/ChainAsset/TransferAsset.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { Token } from "@unionlabs/sdk/schema"
import { Option } from "effect"

type Props = {
  onSelect: () => void
}

const { onSelect }: Props = $props()

// Get the destination tokens to find the actual token objects for each representation
const destinationTokens = $derived(
  transferData.destinationChain.pipe(
    Option.flatMap((dc) => tokensStore.getData(dc.universal_chain_id)),
  ),
)

// Convert representations to actual token objects
const representationTokens = $derived.by(() => {
  return Option.all([
    transferData.representations,
    destinationTokens,
  ]).pipe(
    Option.map(([representations, tokens]) => {
      return representations
        .map(wrapping => {
          // Find the token that matches this representation's unwrapped denom
          const token = tokens.find(t => t.denom === wrapping.unwrapped_denom)
          return token ? { token, wrapping } : null
        })
        .filter((item): item is { token: Token; wrapping: any } => item !== null)
    }),
    Option.getOrElse(() => [] as Array<{ token: Token; wrapping: any }>)
  )
})

function selectRepresentation(tokenData: { token: Token; wrapping: any }) {
  // Set the quote token to the selected representation's denom
  transferData.raw.updateField("quoteToken", tokenData.token.denom)
  onSelect()
}
</script>

<div class="h-full flex flex-col relative">
  <div class="overflow-y-auto flex-grow">
    <div class="w-full">
      {#if Option.isNone(transferData.destinationChain)}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          Please select a destination chain first
        </div>
      {:else if Option.isNone(transferData.baseToken)}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          Please select a base token first
        </div>
      {:else if Option.isNone(transferData.representations)}
        <div>
          {#each Array(3) as _}
            <div class="flex items-center w-full px-4 py-2 border-b border-zinc-700">
              <div class="flex-1 min-w-0">
                <div class="mb-1">
                  <Skeleton
                    class="h-4 w-24"
                    randomWidth={true}
                  />
                </div>
                <Skeleton
                  class="h-3 w-32"
                  randomWidth={true}
                />
              </div>
              <div class="ml-2">
                <Skeleton class="h-4 w-4" />
              </div>
            </div>
          {/each}
        </div>
      {:else if representationTokens.length === 0}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          No representations found for this destination
        </div>
      {:else}
        <div class="p-4">
          <div class="mb-4">
            <h3 class="text-lg font-semibold text-zinc-100 mb-2">
              Choose Representation
            </h3>
            <p class="text-sm text-zinc-400">
              This token has multiple representations on {transferData.destinationChain.value?.display_name}. 
              Select which one you'd like to receive.
            </p>
          </div>
          
          <div class="flex flex-col gap-1">
            {#each representationTokens as tokenData, index}
              {#key tokenData.token.denom}
                <div class="relative">
                  <TransferAsset
                    chain={transferData.destinationChain.value}
                    token={tokenData.token}
                    selectAsset={() => selectRepresentation(tokenData)}
                    {index}
                  />
                  
                  <!-- Show additional info about the wrapping -->
                  <div class="absolute right-4 top-1/2 -translate-y-1/2 text-xs text-zinc-500">
                    Channel {tokenData.wrapping.destination_channel_id}
                    {#if tokenData.wrapping.wrapper}
                      • {tokenData.wrapping.wrapper}
                    {/if}
                  </div>
                </div>
              {/key}
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
