<script lang="ts">
import type { Readable } from "svelte/store"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { truncate } from "$lib/utilities/format.ts"
import { formatUnits } from "viem"
import { Button } from "$lib/components/ui/button"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"

interface Props {
  stores: {
    rawIntents: RawIntentsStore
    context: Readable<ContextStore>
  }
  rotateTo: (face: CubeFaces) => void
}

export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]

let { rawIntents, context } = stores

$: sortedAssets = [...($context.assetsList ?? [])].sort((a, b) => {
  if (a.isSupported !== b.isSupported) {
    return a.isSupported ? -1 : 1
  }
  return Number(b.balance.balance - a.balance.balance)
})

function setAsset(address: string) {
  rawIntents.updateField("asset", address)
  rotateTo("intentFace")
}
</script>

<div class="flex flex-col h-full w-full">
  <div class="text-primary p-2 px-4 flex items-center justify-between border-b-2">
    <span class="font-bold uppercase">Assets</span>
    <button
            class="border-2 h-6 w-6 flex items-center justify-center"
            on:click={() => rotateTo("intentFace")}
    >✕
    </button>
  </div>

  {#if sortedAssets.length}
    <div class="flex-1 overflow-y-auto">
      {#each sortedAssets as asset}
        <div class="pb-2 flex flex-col justify-start">
          <Button
                  variant="ghost"
                  class="px-4 py-2 w-full rounded-none flex justify-between items-center"
                  on:click={() => setAsset(asset.balance.address)}
          >
            <div class:opacity-30={!asset.isSupported}>
              {truncate(asset.symbol, 6)}
            </div>
            <p class:opacity-30={!asset.isSupported}>
              {formatUnits(asset.balance.balance, asset.supportedAsset?.decimals ?? 0)}
            </p>
          </Button>
        </div>
      {/each}
    </div>
  {:else}
    <div class="px-4 p-2">
      <p>No spendable balances</p>
    </div>
  {/if}
</div>