<script lang="ts">
import type { Readable } from "svelte/store"
import { truncate } from "$lib/utilities/format.ts"
import { formatUnits } from "viem"
import { Button } from "$lib/components/ui/button"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import { derived, writable } from "svelte/store"

interface Props {
  stores: {
    rawIntents: RawIntentsStore
    intents: Readable<IntentsStore>
  }
  rotateTo: (face: CubeFaces) => void
}

export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]

let { rawIntents, intents } = stores

const showZeroBalances = writable(false)

$: filteredAssets = derived([intents, showZeroBalances], ([$intents, $showZeroBalances]) =>
  $showZeroBalances
    ? $intents.sourceAssets
    : $intents.sourceAssets.filter(asset => BigInt(asset.balance) > 0n)
)

function setAsset(denom: string) {
  rawIntents.updateField("asset", denom)
  rotateTo("intentFace")
}

function toggleZeroBalances() {
  showZeroBalances.update(value => !value)
}
</script>

<div class="flex flex-col h-full w-full">
  <div class="text-primary p-2 px-4 flex items-center justify-between border-b-2">
    <div class="flex items-center gap-2">
      <span class="font-bold uppercase">Assets</span>
      <button
              class="text-xs border px-2 py-1 rounded"
              on:click={toggleZeroBalances}
      >
        {$showZeroBalances ? 'Hide' : 'Show'} Zero Balances
      </button>
    </div>
    <button
            class="border-2 h-6 w-6 flex items-center justify-center"
            on:click={() => rotateTo("intentFace")}
    >✕
    </button>
  </div>

  {#if $filteredAssets.length}
    <div class="flex-1 overflow-y-auto">
      {#each $filteredAssets as asset (asset)}
        <div class="pb-2 flex flex-col justify-start">
          <Button
                  variant="ghost"
                  class="px-4 py-2 w-full rounded-none flex justify-between items-center"
                  on:click={() => setAsset(asset.metadata.denom)}
          >
            <div class:opacity-30={asset.metadata.metadata_level === "none"}>
              {truncate(asset.metadata.display_symbol || asset.metadata.denom, 6)}
            </div>
            <p class:opacity-30={asset.metadata.metadata_level === "none"}>
              {formatUnits(BigInt(asset.balance), asset.metadata.decimals ?? 0)}
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