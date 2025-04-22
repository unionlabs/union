<script lang="ts">
import { Array as Arr, flow, Match, Option, pipe, Data } from "effect"
import { chains } from "$lib/stores/chains.svelte.ts"
import { cn } from "$lib/utils"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { UniversalChainId } from "@unionlabs/sdk/schema"
import { chainLogoMap } from "$lib/constants/chain-logos.ts"
import { MODE } from "$lib/constants/config"
import { signingMode } from "$lib/transfer/signingMode.svelte"
import type { Tokens } from "@unionlabs/sdk/schema"
import { log } from "effect/Console"

type Props = {
  type: "source" | "destination"
  onSelect: () => void
}

const { type, onSelect }: Props = $props()

// Chain configuration
const TESTNET_CHAINS: Array<UniversalChainId> = [
  UniversalChainId.make("ethereum.11155111"),
  UniversalChainId.make("corn.21000001"),
  UniversalChainId.make("bob.808813"),
  UniversalChainId.make("babylon.bbn-test-5")
]

const MAINNET_CHAINS: Array<UniversalChainId> = [
  UniversalChainId.make("bob.60808"),
  UniversalChainId.make("corn.21000000"),
  UniversalChainId.make("babylon.bbn-1"),
  UniversalChainId.make("ethereum.1")
]

type ChainWithRateLimit = [Chain, boolean]

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

/**
 * Filters chains based on the current environment (testnet/mainnet)
 */
const filterByEnvironment = (chains: ReadonlyArray<Chain>) => {
  const allowedChains = MODE === "testnet" ? TESTNET_CHAINS : MAINNET_CHAINS
  return chains.filter(chain => allowedChains.includes(chain.universal_chain_id))
}

/**
 * Filters chains based on the current signing mode
 * In multi mode, only cosmos chains are shown for source selection
 */
const filterBySigningMode = (chains: Array<Chain>) =>
  pipe(
    Match.value(signingMode.mode).pipe(
      Match.when("single", () => chains),
      Match.when("multi", () => {
        if (type === "source") {
          return chains.filter(chain => chain.rpc_type === "cosmos")
        }
        return chains
      }),
      Match.exhaustive
    )
  )

type ChainStatus = {
  isSelected: boolean
  isSourceChain: boolean
  isDisabled: boolean
  isRateLimited: boolean
  hasRoute: boolean
}

function getChainStatus(chain: Chain, isRateLimited: boolean): ChainStatus {
  const isSourceChain = type === "destination" && transferData.raw.source === chain.chain_id
  const hasRoute =
    type === "destination" &&
    pipe(
      transferData.destinationChains,
      Option.map(goodXs => goodXs.map(x => x.chain_id).includes(chain.chain_id)),
      Option.getOrElse(() => false)
    )
  const isSelected =
    type === "source"
      ? transferData.raw.source === chain.chain_id
      : transferData.raw.destination === chain.chain_id
  const isDisabled =
    type === "destination"
      ? isSourceChain || !hasRoute || isRateLimited // isRateLimited now means "no bucket"
      : false

  return { isSelected, isSourceChain, isDisabled, isRateLimited, hasRoute }
}

/**
 * Checks if destination chains have rate limiting (bucket) for the wrapped version of the base token
 */
const filterByTokenBucket = (chains: Array<Chain>): Array<ChainWithRateLimit> => {
  if (type !== "destination" || Option.isNone(transferData.baseToken)) {
    return chains.map(chain => [chain, false])
  }

  const baseToken = transferData.baseToken.value
  return chains.map(chain => {
    const chainTokens = tokensStore.getData(chain.universal_chain_id)
    if (Option.isNone(chainTokens)) {
      return [chain, false]
    }

    const token = chainTokens.value.find(t =>
      t.wrapping.some(w => w.unwrapped_denom === baseToken.denom)
    )

    return [chain, token?.bucket == null] // No bucket means disabled
  })
}

// Apply all chain filters in sequence
const filteredChains = $derived(
  pipe(chains.data, Option.map(flow(filterByEnvironment, filterBySigningMode, filterByTokenBucket)))
)
</script>

<div class="p-4">
  {#if Option.isSome(filteredChains)}
    {@const chainss = filteredChains.value}
    <div class="grid grid-cols-3 gap-2">
      {#each chainss as [chain, isRateLimited]}
        {@const status = getChainStatus(chain, isRateLimited)}
        {@const chainLogo = chain.universal_chain_id ? chainLogoMap.get(chain.universal_chain_id) : null}

        <button
          class={cn(
            "flex flex-col items-center gap-2 justify-start px-2 py-4 rounded-md transition-colors",
            status.isSelected
              ? "bg-zinc-900 hover:bg-zinc-800 ring-1 ring-babylon-orange"
              : status.isDisabled
                ? "bg-zinc-900 opacity-50 cursor-not-allowed"
                : "bg-zinc-900 hover:bg-zinc-800 cursor-pointer"
          )}
          onclick={() => !status.isDisabled && selectChain(chain)}
          disabled={status.isDisabled}
        >
          {#if chainLogo?.color}
            <span class="w-10 h-10 flex items-center justify-center overflow-hidden">
              <img src={chainLogo.color} alt="" />
            </span>
          {/if}

          <span class="text-xs text-center truncate w-fit">{chain.display_name}</span>

          {#if status.isSourceChain}
            <span class="text-xs text-sky-400 -mt-2">Source Chain</span>
          {/if}
          {#if status.isRateLimited && !status.isSourceChain}
            <span class="text-xs text-red-400 -mt-2">Rate Limited</span>
          {/if}
          {#if type === "destination" && !status.hasRoute && !status.isSourceChain}
            <span class="text-xs text-yellow-400 -mt-2">No route</span>
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

