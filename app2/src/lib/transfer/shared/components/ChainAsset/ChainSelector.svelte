<script lang="ts">
import { Match, Option, pipe, Tuple } from "effect"
import { chains } from "$lib/stores/chains.svelte.ts"
import { cn } from "$lib/utils"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { Chain, Token, TokenWrapping } from "@unionlabs/sdk/schema"
import { chainLogoMap } from "$lib/constants/chain-logos.ts"
import { MODE } from "$lib/constants/config"
import { signingMode } from "$lib/transfer/signingMode.svelte"
import { MAINNET_CHAINS, TESTNET_CHAINS } from "$lib/constants/chains.ts"

type ChainSelectorStatus = {
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

const { type, onSelect }: Props = $props()

type ChainWithAvailability = ReturnType<typeof Tuple.make<[Chain, boolean]>>

function updateSelectedChain(chain: Chain) {
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

const isValidRoute = (chain: Chain) =>
  pipe(
    Match.value(type).pipe(
      Match.when("source", () => true),
      Match.when("destination", () =>
        pipe(
          transferData.destinationChains,
          Option.map(goodXs => goodXs.map(x => x.chain_id).includes(chain.chain_id)),
          Option.getOrElse(() => false)
        )
      ),
      Match.exhaustive
    )
  )

function getChainStatus(chain: Chain, hasBucket: boolean): ChainSelectorStatus {
  const isSourceChain = type === "destination" && transferData.raw.source === chain.chain_id

  return pipe(
    Match.value(type).pipe(
      Match.when("source", () => ({
        isSelected: transferData.raw.source === chain.chain_id,
        isSourceChain: false,
        isDisabled: false,
        hasBucket,
        hasRoute: true
      })),
      Match.when("destination", () => ({
        isSelected: transferData.raw.destination === chain.chain_id,
        isSourceChain,
        isDisabled: isSourceChain || !isValidRoute(chain) || !hasBucket,
        hasBucket,
        hasRoute: isValidRoute(chain)
      })),
      Match.exhaustive
    )
  )
}

const findTokenWithBucket = (
  tokenList: ReadonlyArray<Token>,
  predicate: (token: Token) => boolean
) =>
  pipe(
    tokenList.find(predicate),
    Option.fromNullable,
    Option.map(token => token.bucket != null),
    Option.getOrElse(() => false)
  )

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

  return pipe(
    Option.fromNullable(maybeUnwrapped),
    Option.match({
      onSome: unwrapped =>
        findTokenWithBucket(
          tokenList,
          t => t.denom.toLowerCase() === unwrapped.unwrapped_denom.toLowerCase()
        ),
      onNone: () =>
        findTokenWithBucket(tokenList, t =>
          t.wrapping.some(
            (w: TokenWrapping) =>
              w.unwrapped_denom.toLowerCase() === baseDenom &&
              w.unwrapped_chain.universal_chain_id === sourceChain.universal_chain_id &&
              w.wrapped_chain.universal_chain_id === destinationChain.universal_chain_id
          )
        )
    })
  )
}

const filterChainsByTokenAvailability = (chains: Array<Chain>): Array<ChainWithAvailability> =>
  pipe(
    Match.value(type).pipe(
      Match.when("source", () => chains.map(chain => Tuple.make(chain, false))),
      Match.when("destination", () =>
        pipe(
          Option.all({
            baseToken: transferData.baseToken,
            sourceChain: transferData.sourceChain
          }),
          Option.match({
            onNone: () => chains.map(chain => Tuple.make(chain, false)),
            onSome: ({ baseToken, sourceChain }) =>
              chains.map(destinationChain => {
                const tokens = tokensStore.getData(destinationChain.universal_chain_id)
                return Option.match(tokens, {
                  onNone: () => Tuple.make(destinationChain, false),
                  onSome: tokenList =>
                    Tuple.make(
                      destinationChain,
                      hasTokenBucket(destinationChain, tokenList, baseToken, sourceChain)
                    )
                })
              })
          })
        )
      ),
      Match.exhaustive
    )
  )

const filteredChains = $derived(
  pipe(
    chains.data,
    Option.map(allChains => pipe(allChains, filterByEnvironment, filterBySigningMode)),
    Option.map(filterChainsByTokenAvailability)
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
          onclick={() => !status.isDisabled && updateSelectedChain(chain)}
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
