<script lang="ts">
import { chainLogoMap } from "$lib/constants/chain-logos.ts"
import { DISABLED_CHAINS } from "$lib/constants/disabled-chains.ts"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { uiStore } from "$lib/stores/ui.svelte.ts"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { signingMode } from "$lib/transfer/signingMode.svelte"
import { cn } from "$lib/utils"
import type { Chain, Token, TokenWrapping } from "@unionlabs/sdk/schema"
import { Match, Option, pipe, Tuple } from "effect"

type Props = {
  type: "source" | "destination"
  onSelect: () => void
}

const { type, onSelect }: Props = $props()

type ChainWithAvailability = ReturnType<typeof Tuple.make<[Chain, boolean]>>

const updateSelectedChain = (chain: Chain) => {
  pipe(
    Match.value(type).pipe(
      Match.when("destination", () => {
        if (chain.chain_id === transferData.raw.source) {
          return
        }
        transferData.raw.updateField(type, chain.chain_id)
      }),
      Match.when("source", () => {
        transferData.raw.updateField(type, chain.chain_id)
        if (transferData.raw.destination === chain.chain_id) {
          transferData.raw.updateField("destination", "")
        }
      }),
      Match.exhaustive,
    ),
  )
  onSelect()
}

const filterBySigningMode = (chains: Array<Chain>) =>
  signingMode.mode === "multi" && type === "source"
    ? chains.filter(chain => chain.rpc_type === "cosmos")
    : chains

const isValidRoute = (chain: Chain) =>
  type === "source"
  || pipe(
    transferData.destinationChains,
    Option.map(goodXs => goodXs.map(x => x.chain_id).includes(chain.chain_id)),
    Option.getOrElse(() => false),
  )

const getChainStatus = (chain: Chain, hasBucket: boolean) => {
  const isSourceChain = type === "destination" && transferData.raw.source === chain.chain_id
  const isDisabledChain = chain.universal_chain_id
    && DISABLED_CHAINS.includes(chain.universal_chain_id)

  return pipe(
    Match.value(type).pipe(
      Match.when("source", () => ({
        isSelected: transferData.raw.source === chain.chain_id,
        isSourceChain: false,
        isDisabled: isDisabledChain,
        hasBucket,
        hasRoute: true,
      })),
      Match.when("destination", () => ({
        isSelected: transferData.raw.destination === chain.chain_id,
        isSourceChain,
        isDisabled: isSourceChain || !isValidRoute(chain) || !hasBucket || isDisabledChain,
        hasBucket,
        hasRoute: isValidRoute(chain),
      })),
      Match.exhaustive,
    ),
  )
}

const findTokenWithBucket = (
  tokenList: ReadonlyArray<Token>,
  predicate: (token: Token) => boolean,
) =>
  pipe(
    tokenList.find(predicate),
    Option.fromNullable,
    Option.map(token => token.bucket != null),
    Option.getOrElse(() => false),
  )

const hasTokenBucket = (
  destinationChain: Chain,
  tokenList: ReadonlyArray<Token>,
  baseToken: Token,
  sourceChain: Chain,
) => {
  const baseDenom = baseToken.denom.toLowerCase()

  const maybeUnwrapped = baseToken.wrapping.find(
    (w: TokenWrapping) =>
      w.wrapped_chain.universal_chain_id === sourceChain.universal_chain_id
      && w.unwrapped_chain.universal_chain_id === destinationChain.universal_chain_id,
  )

  return pipe(
    Option.fromNullable(maybeUnwrapped),
    Option.match({
      onSome: unwrapped =>
        findTokenWithBucket(
          tokenList,
          t => t.denom.toLowerCase() === unwrapped.unwrapped_denom.toLowerCase(),
        ),
      onNone: () =>
        findTokenWithBucket(tokenList, t =>
          t.wrapping.some(
            (w: TokenWrapping) =>
              w.unwrapped_denom.toLowerCase() === baseDenom
              && w.unwrapped_chain.universal_chain_id === sourceChain.universal_chain_id
              && w.wrapped_chain.universal_chain_id === destinationChain.universal_chain_id,
          )),
    }),
  )
}

const filterChainsByTokenAvailability = (
  chains: Array<Chain>,
  filterWhitelist: boolean,
): Array<ChainWithAvailability> =>
  pipe(
    Match.value(type).pipe(
      Match.when("source", () => chains.map(chain => Tuple.make(chain, false))),
      Match.when("destination", () =>
        pipe(
          Option.all({
            baseToken: transferData.baseToken,
            sourceChain: transferData.sourceChain,
          }),
          Option.match({
            onNone: () => chains.map(chain => Tuple.make(chain, false)),
            onSome: ({ baseToken, sourceChain }) =>
              chains.map(destinationChain => {
                // For testnet chains, we always mark them as available (hasBucket=true) to allow testing
                if (destinationChain.testnet === true) {
                  return Tuple.make(destinationChain, true)
                }
                if (!filterWhitelist) {
                  return Tuple.make(destinationChain, true)
                }
                const tokens = tokensStore.getData(destinationChain.universal_chain_id)
                return Option.match(tokens, {
                  onNone: () => Tuple.make(destinationChain, false),
                  onSome: tokenList =>
                    Tuple.make(
                      destinationChain,
                      hasTokenBucket(destinationChain, tokenList, baseToken, sourceChain),
                    ),
                })
              }),
          }),
        )),
      Match.exhaustive,
    ),
  )

const filteredChains = $derived(
  pipe(
    // Now use transferData.filteredChains which already includes edition filtering
    transferData.filteredChains,
    Option.map(allChains =>
      pipe(
        allChains,
        filterBySigningMode,
        chains => filterChainsByTokenAvailability(chains, uiStore.filterWhitelist),
        chainWithAvailability => {
          return chainWithAvailability.sort((a, b) => {
            const [chainA, hasBucketA] = a
            const [chainB, hasBucketB] = b
            
            const statusA = getChainStatus(chainA, hasBucketA)
            const statusB = getChainStatus(chainB, hasBucketB)

            // For destination selector: source chain first, then selectable, then disabled
            if (type === "destination") {
              // Source chain always goes first
              if (statusA.isSourceChain && !statusB.isSourceChain) {
                return -1
              }
              if (!statusA.isSourceChain && statusB.isSourceChain) {
                return 1
              }
              
              // If neither is source chain, selectable chains go before disabled
              if (!statusA.isSourceChain && !statusB.isSourceChain) {
                if (!statusA.isDisabled && statusB.isDisabled) {
                  return -1
                }
                if (statusA.isDisabled && !statusB.isDisabled) {
                  return 1
                }
              }
            } else {
              // For source selector: just selectable first, then disabled
              if (!statusA.isDisabled && statusB.isDisabled) {
                return -1
              }
              if (statusA.isDisabled && !statusB.isDisabled) {
                return 1
              }
            }

            // If both have same status, maintain original order
            return 0
          })
        },
      )
    ),
  ),
)
</script>

<div>
  {#if Option.isSome(filteredChains)}
    {@const chainss = filteredChains.value}
    <div class="relative">
              <div class="p-4 grid grid-cols-2 gap-4 max-h-[459px] overflow-y-auto scrollbar-thin scrollbar-track-zinc-900 scrollbar-thumb-zinc-700">
        {#each chainss as chainWithAvailability, index}
          {@const [chain, hasBucket] = chainWithAvailability}
          {@const status = getChainStatus(chain, hasBucket)}
          {@const chainLogo = chain.universal_chain_id
          ? chainLogoMap.get(chain.universal_chain_id)
          : null}

          <button
            style="animation-delay: {index * 50}ms;"
            class={cn(
              "group relative flex flex-col items-center gap-3 justify-start p-3 rounded-xl transition-all duration-200 min-h-[130px] border",
              "transform hover:scale-[1.02] active:scale-[0.98]",
              "animate-fade-in-up opacity-0",
              status.isSelected
                ? "bg-gradient-to-br from-accent/20 to-accent/10 border-accent/50 shadow-lg shadow-accent/20 ring-2 ring-accent/30"
                : status.isDisabled
                ? "bg-gradient-to-br from-zinc-900/50 to-zinc-800/30 border-zinc-700/30 opacity-60 cursor-not-allowed"
                : "bg-gradient-to-br from-zinc-900 to-zinc-800/80 border-zinc-700/50 hover:border-accent/30 hover:shadow-md hover:shadow-accent/10 cursor-pointer",
            )}
            onclick={() => !status.isDisabled && updateSelectedChain(chain)}
            disabled={status.isDisabled}
                    >      

            <!-- Chain logo with enhanced styling -->
            <div class={cn(
              "relative w-12 h-12 flex items-center justify-center rounded-full transition-all duration-200",
              "bg-gradient-to-br from-zinc-800 to-zinc-700 border border-zinc-600/50",
              !status.isDisabled && "group-hover:scale-110 group-hover:shadow-lg"
            )}>
              {#if chainLogo?.color}
                <img
                  src={chainLogo.color}
                  alt={chain.display_name}
                  class="w-8 h-8 object-contain"
                />
              {:else}
                <!-- Fallback icon for chains without logos -->
                <div class="w-6 h-6 bg-gradient-to-br from-accent/60 to-accent/80 rounded-full"></div>
              {/if}
              
              <!-- Testnet badge overlay on logo -->
              {#if chain.testnet}
                <div class="absolute -top-1 -right-1 w-4 h-4 bg-amber-500 rounded-full flex items-center justify-center border border-amber-400 shadow-sm">
                  <svg class="w-2 h-2 text-amber-100" fill="currentColor" viewBox="0 0 16 16">
                    <path d="M9.972 2.508a.5.5 0 0 0-.16-.556l-.178-.129a5.009 5.009 0 0 0-2.076-.783C6.215.862 4.504 1.229 2.84 3.133H1.786a.5.5 0 0 0-.354.147L.146 4.567a.5.5 0 0 0 0 .706l2.571 2.579a.5.5 0 0 0 .708 0l1.286-1.29a.5.5 0 0 0 .146-.353V5.57l8.387 8.873A.5.5 0 0 0 14 14.5l1.5-1.5a.5.5 0 0 0 .017-.689l-9.129-8.63c.747-.456 1.772-.839 3.112-.839a.5.5 0 0 0 .472-.334z"/>
                  </svg>
                </div>
              {/if}
            </div>

            <!-- Chain name with better typography -->
            <div class="text-center flex-1 flex flex-col justify-center pb-8">
              <span class={cn(
                "text-sm font-medium leading-tight",
                status.isSelected ? "text-white" : "text-zinc-200",
                status.isDisabled && "text-zinc-400"
              )}>
                {chain.display_name.split(' ')[0]}
              </span>
            </div>

            <!-- Status indicators with better visual hierarchy -->
            <div class="absolute bottom-2 left-2 right-2">
              {#if status.isSourceChain}
                <div class="flex items-center justify-center gap-1.5 px-2 py-1 bg-sky-500/20 text-sky-300 text-xs rounded-lg border border-sky-500/30">
                  <div class="w-1.5 h-1.5 bg-sky-400 rounded-full animate-pulse"></div>
                  Source
                </div>
              {:else if chain.universal_chain_id && DISABLED_CHAINS.includes(chain.universal_chain_id)}
                <div class="flex items-center justify-center gap-1.5 px-2 py-1 bg-red-500/20 text-red-300 text-xs rounded-lg border border-red-500/30">
                  <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M13.477 14.89A6 6 0 015.11 6.524l8.367 8.368zm1.414-1.414L6.524 5.11a6 6 0 018.367 8.367zM18 10a8 8 0 11-16 0 8 8 0 0116 0z" clip-rule="evenodd"/>
                  </svg>
                  Disabled
                </div>
              {:else if type === "destination" && !status.hasRoute && !status.isSourceChain}
                <div class="flex items-center justify-center gap-1.5 px-2 py-1 bg-orange-500/20 text-orange-300 text-xs rounded-lg border border-orange-500/30">
                  <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                  </svg>
                  No Route
                </div>
              {:else if type === "destination" && !status.hasBucket && status.hasRoute && !status.isSourceChain}
                <div class="flex items-center justify-center gap-1.5 px-2 py-1 bg-yellow-500/20 text-yellow-300 text-xs rounded-lg border border-yellow-500/30">
                  <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/>
                  </svg>
                  Not Whitelisted
                </div>
              {:else}
                <!-- Available/selectable chains get a positive indicator -->
                <div class="flex items-center justify-center gap-1.5 px-2 py-1 bg-green-500/20 text-green-300 text-xs rounded-lg border border-green-500/30">
                  <div class="w-1.5 h-1.5 bg-green-400 rounded-full"></div>
                  Route open
                </div>
              {/if}
            </div>

            <!-- Subtle hover effect overlay -->
            {#if !status.isDisabled}
              <div class="absolute inset-0 bg-gradient-to-br from-accent/0 to-accent/5 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none"></div>
            {/if}
          </button>
        {/each}
      </div>
    </div>
    
    <!-- Enhanced gradient fade with better styling -->
    <!-- <div class="absolute bottom-0 left-0 right-0 h-20 bg-gradient-to-t from-zinc-925 via-zinc-925/80 to-transparent pointer-events-none"></div> -->
  {:else}
    <div class="py-8 text-center">
      <div class="inline-flex items-center gap-3 px-4 py-3 bg-zinc-900/50 rounded-xl border border-zinc-700/50">
        <div class="w-5 h-5 border-2 border-accent border-t-transparent rounded-full animate-spin"></div>
        <span class="text-zinc-400 font-medium">Loading chains...</span>
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes fade-in-up {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
  
  .animate-fade-in-up {
    animation: fade-in-up 0.6s ease-out forwards;
  }
</style>
