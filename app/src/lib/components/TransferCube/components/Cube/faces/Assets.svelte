<script lang="ts">
import { type Readable, writable } from "svelte/store"
import type { CubeFaces } from "$lib/components/TransferCube/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferCube/transfer/raw-intents.ts"
import Token from "$lib/components/token.svelte"
import type { Chain } from "$lib/types"
import type { Intents } from "$lib/components/TransferCube/transfer/types.ts"
import InlineLoadingDots from "$lib/components/InlineLoadingDots.svelte"

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
    ? intents.baseTokens.filter(
        token =>
          token.balance?.kind === "loading" ||
          (token.balance?.kind === "balance" &&
            token.balance.amount !== null &&
            token.balance.amount !== "0")
      )
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
  <div class="flex-1 relative">
    <div class="absolute inset-0 overflow-y-auto overflow-x-hidden -webkit-overflow-scrolling-touch">
      {#each filteredTokens as token}
        <button
                class="px-2 py-1 flex flex-col hover:bg-neutral-400 dark:hover:bg-neutral-800 text-sm justify-start items-start w-full overflow-x-auto"
                on:click={() => setAsset(token.denom)}
        >
          {#if token.balance && token.balance.kind === "balance"}
            <div class="text-sm flex justify-start items-center">
              <Token stackedView highlightEnabled={false} chainId={$rawIntents.source} denom={token.denom} amount={token.balance.amount || 0} {chains}/>
            </div>
          {:else}
            <div>
              <Token stackedView highlightEnabled={false} chainId={$rawIntents.source} denom={token.denom} {chains}/>
            </div>
            <div class="text-muted-foreground text-xs self-start text-left">
            {#if !token.balance}
              No balance fetched.
            {:else if token.balance.kind === "error"}
              <div class="text-red-500">
                Error loading balance:
                {token.balance.error}
              </div>
            {:else if token.balance.kind === "loading"}
              <InlineLoadingDots>Loading Balance</InlineLoadingDots>
            {/if}
            </div>
          {/if}
        </button>
      {/each}
    </div>
  </div>
</div>
