<script lang="ts">
import { Match, Option, pipe, Tuple } from "effect"
import { chains } from "$lib/stores/chains.svelte.ts"
import { cn } from "$lib/utils"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import { chainLogoMap } from "$lib/constants/chain-logos.ts"
import { MODE } from "$lib/constants/config"
import { signingMode } from "$lib/transfer/signingMode.svelte"
import { MAINNET_CHAINS, TESTNET_CHAINS } from "$lib/constants/chains.ts"

type ChainStatus = {
  isSelected: boolean
  isSourceChain: boolean
  isDisabled: boolean
  hasBucket: boolean
  hasRoute: boolean
}

type Props = {
  type: "source" | "destination"
  onSelect: () => void
}

type TokenWrapping = {
  wrapped_chain: { universal_chain_id: string }
  unwrapped_chain: { universal_chain_id: string }
  unwrapped_denom: string
}

const { type, onSelect }: Props = $props()

type ChainWithAvailability = ReturnType<typeof Tuple.make<[Chain, boolean]>>

function selectChain(chain: Chain) {
  if (type === "destination" && chain.chain_id === transferData.raw.source) {
    return
  }

  transferData.raw.updateField(type, chain.chain_id)

  if (type === "source" && transferData.raw.destination === chain.chain_id) {
    transferData.raw.updateField("destination", "")
  }

  onSelect()
}

const filterByEnvironment = (chains: ReadonlyArray<Chain>) => {
  const allowedChains = MODE === "testnet" ? TESTNET_CHAINS : MAINNET_CHAINS
  return chains.filter(chain => allowedChains.includes(chain.universal_chain_id))
}

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

const hasRoute = (chain: Chain) =>
  type === "destination" &&
  pipe(
    transferData.destinationChains,
    Option.map(goodXs => goodXs.map(x => x.chain_id).includes(chain.chain_id)),
    Option.getOrElse(() => false)
  )

function getChainStatus(chain: Chain, hasBucket: boolean): ChainStatus {
  const isSourceChain = type === "destination" && transferData.raw.source === chain.chain_id
  const isSelected =
    type === "source"
      ? transferData.raw.source === chain.chain_id
      : transferData.raw.destination === chain.chain_id
  const isDisabled =
    type === "destination" ? isSourceChain || !hasRoute(chain) || !hasBucket : false

  return {
    isSelected,
    isSourceChain,
    isDisabled,
    hasBucket,
    hasRoute: hasRoute(chain)
  }
}

const hasTokenBucket = (
  destinationChain: Chain,
  tokenList: ReadonlyArray<Token>,
  baseToken: Token,
  sourceChain: Chain
) => {
  const baseDenom = baseToken.denom.toLowerCase()

  const maybeUnwrapped = baseToken.wrapping.find(
    (w: TokenWrapping) =>
      w.wrapped_chain.universal_chain_id === sourceChain.universal_chain_id &&
      w.unwrapped_chain.universal_chain_id === destinationChain.universal_chain_id
  )

  if (maybeUnwrapped) {
    const destToken = tokenList.find(
      t => t.denom.toLowerCase() === maybeUnwrapped.unwrapped_denom.toLowerCase()
    )
    return destToken?.bucket != null
  }

  const reverseMatch = tokenList.find(t =>
    t.wrapping.some(
      (w: TokenWrapping) =>
        w.unwrapped_denom.toLowerCase() === baseDenom &&
        w.unwrapped_chain.universal_chain_id === sourceChain.universal_chain_id &&
        w.wrapped_chain.universal_chain_id === destinationChain.universal_chain_id
    )
  )

  return reverseMatch?.bucket != null
}

const filterByTokenAvailability = (chains: Array<Chain>): Array<ChainWithAvailability> => {
  if (
    type !== "destination" ||
    Option.isNone(transferData.baseToken) ||
    Option.isNone(transferData.sourceChain)
  ) {
    return chains.map(chain => Tuple.make(chain, false))
  }

  const baseToken = transferData.baseToken.value
  const sourceChain = transferData.sourceChain.value

  return chains.map(destinationChain => {
    const tokens = tokensStore.getData(destinationChain.universal_chain_id)
    if (Option.isNone(tokens)) return Tuple.make(destinationChain, false)

    return Tuple.make(
      destinationChain,
      hasTokenBucket(destinationChain, tokens.value, baseToken, sourceChain)
    )
  })
}

const filteredChains = $derived(
  pipe(
    chains.data,
    Option.map(allChains => pipe(allChains, filterByEnvironment, filterBySigningMode)),
    Option.map(filterByTokenAvailability)
  )
)
</script>

<div class="p-4">
  {#if Option.isSome(filteredChains)}
    {@const chainss = filteredChains.value}
    <div class="grid grid-cols-3 gap-2">
      {#each chainss as chainWithAvailability}
        {@const [chain, hasBucket] = chainWithAvailability}
        {@const status = getChainStatus(chain, hasBucket)}
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
              <img src={chainLogo.color} alt=""/>
            </span>
          {/if}

          <span class="text-xs text-center truncate w-fit">{chain.display_name}</span>

          {#if status.isSourceChain}
            <span class="text-xs text-sky-400 -mt-2">Source Chain</span>
          {/if}
          {#if type === "destination" && !status.hasBucket && !status.isSourceChain}
            <span class="text-xs text-red-400 -mt-2">No bucket</span>
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
