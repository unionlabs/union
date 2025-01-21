<script lang="ts">
import type { Readable } from "svelte/store"
import { truncate } from "$lib/utilities/format.ts"
import { formatUnits } from "viem"
import { Button } from "$lib/components/ui/button"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import { derived, writable } from "svelte/store"
import Token from "$lib/components/token.svelte"
import type { Chain } from "$lib/types"

interface Props {
  stores: {
    rawIntents: RawIntentsStore
    intents: Readable<IntentsStore>
  }
  rotateTo: (face: CubeFaces) => void
}

export let chains: Array<Chain>
export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]

let { rawIntents, intents } = stores

let sortedTokens = derived([intents], ([$intents]) =>
  $intents.baseTokens.toSorted((a, b) => Number(BigInt(b.balance) - BigInt(a.balance)))
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
    </div>
    <button
            class="border-2 h-6 w-6 flex items-center justify-center"
            on:click={() => rotateTo("intentFace")}
    >✕
    </button>
  </div>

  <div class="flex flex-col overflow-y-auto">
  {#each $sortedTokens as token}
    <button
            class="px-2 py-1 hover:bg-neutral-400 dark:hover:bg-neutral-800 text-md flex justify-start items-center"
            on:click={() => setAsset(token.denom)}
    >
      <Token chainId={$rawIntents.source} denom={token.denom} amount={token.balance} {chains}/>
    </button>
  {/each}
  </div>
</div>
