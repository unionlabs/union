<script lang="ts">
import { type Readable, writable } from "svelte/store"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import Token from "$lib/components/token.svelte"
import type { Chain } from "$lib/types"
import type { Intents } from "$lib/components/TransferFrom/transfer/types.ts"

interface Props {
  rawIntents: RawIntentsStore
  intents: Intents
  validation: Readable<any>
  rotateTo: (face: CubeFaces) => void
}

export let chains: Array<Chain>
export let rawIntents: Props["rawIntents"]
export let intents: Props["intents"]
export let rotateTo: Props["rotateTo"]

function setAsset(denom: string) {
  rawIntents.updateField("asset", denom)
  rawIntents.set({ asset: denom })
  rotateTo("intentFace")
}

const hideZeroBalances = writable(true)

$: filteredTokens =
  $hideZeroBalances && intents.baseTokens
    ? intents.baseTokens.filter(token => token.balance !== "0")
    : (intents.baseTokens ?? [])
</script>

<div class="flex flex-col h-full w-full">
  <div class="text-primary p-2 px-4 flex items-center justify-between border-b-2">
    <div class="flex items-center gap-2">
      <span class="font-bold uppercase">Assets</span>
      <label class="flex items-center gap-1 text-xs">
        <input
                type="checkbox"
                bind:checked={$hideZeroBalances}
                class="accent-primary"
        />
        Hide zero balances
      </label>
    </div>
    <button
            class="border-2 h-6 w-6 flex items-center justify-center"
            on:click={() => rotateTo("intentFace")}
    >✕</button>
  </div>

  <!-- SCROLL CONTAINER -->
  <div class="flex-1 overflow-y-auto">
    <div class="flex flex-col gap-2 py-2">
      {#each filteredTokens as token}
        <button
                class="px-2 py-1 hover:bg-neutral-400 dark:hover:bg-neutral-800 text-sm flex justify-start items-center w-full"
                on:click={() => setAsset(token.denom)}
        >
          <Token stackedView highlightEnabled={false} chainId={$rawIntents.source} denom={token.denom} amount={token.balance} {chains}/>
        </button>
      {/each}
    </div>
  </div>
</div>
