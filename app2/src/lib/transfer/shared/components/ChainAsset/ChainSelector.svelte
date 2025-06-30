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
import { fade } from "svelte/transition"

type Props = {
  type: "source" | "destination"
  onSelect: () => void
}

const { type, onSelect }: Props = $props()

type ChainWithAvailability = ReturnType<typeof Tuple.make<[Chain, boolean]>>

let topFadeOpacity = $state(0)
let bottomFadeOpacity = $state(1)

function handleScroll(e: Event) {
  const target = e.target as HTMLElement
  // Gradually fade in over first 30px of scroll
  topFadeOpacity = Math.min(target.scrollTop / 100, 1)
  
  // Gradually fade out bottom fade when near bottom
  const scrollFromBottom = target.scrollHeight - target.scrollTop - target.clientHeight
  bottomFadeOpacity = Math.min(scrollFromBottom / 100, 1)
}

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
      <!-- Top gradient fade -->
      {#if topFadeOpacity > 0}
        <div 
          class="absolute top-0 left-0 right-0 h-12 bg-gradient-to-b from-zinc-925 to-transparent pointer-events-none z-10" 
          style="opacity: {topFadeOpacity}"
          transition:fade={{ duration: 150 }}
        ></div>
      {/if}
      
      <div class="p-4 grid grid-cols-2 gap-4 max-h-[459px] overflow-y-auto scrollbar-thin scrollbar-track-zinc-900 scrollbar-thumb-zinc-700" onscroll={handleScroll}>
        {#each chainss as chainWithAvailability, index}
          {@const [chain, hasBucket] = chainWithAvailability}
          {@const status = getChainStatus(chain, hasBucket)}
          {@const chainLogo = chain.universal_chain_id
          ? chainLogoMap.get(chain.universal_chain_id)
          : null}

          <button
            style="animation-delay: {index * 50}ms;"
            class={cn(
              "group relative flex flex-col items-center gap-3 justify-start p-3 rounded transition-all duration-100 min-h-[130px] border",
              "animate-fade-in-up opacity-0",
              status.isSelected
                ? "bg-zinc-900 border-accent text-white"
                : status.isDisabled
                ? "bg-zinc-900/50 border-zinc-800/50 opacity-50 cursor-not-allowed"
                : "bg-zinc-900 border-zinc-800 hover:border-zinc-600 cursor-pointer",
            )}
            onclick={() => !status.isDisabled && updateSelectedChain(chain)}
            disabled={status.isDisabled}
                    >      

            <!-- Chain logo -->
            <div class={cn(
              "relative w-12 h-12 flex items-center justify-center rounded-full transition-all duration-100",
              "bg-zinc-800 border border-zinc-700",
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
              
              <!-- Testnet indicator -->
              {#if chain.testnet}
                <div class="absolute -top-0.5 -right-0.5 w-3 h-3 bg-amber-400 rounded-full border border-zinc-900"></div>
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

            <!-- Status indicators -->
            <div class="absolute bottom-2 left-2 right-2">
              {#if status.isSourceChain}
                <div class="text-center text-xs text-accent font-mono">
                  SOURCE
                </div>
              {:else if chain.universal_chain_id && DISABLED_CHAINS.includes(chain.universal_chain_id)}
                <div class="text-center text-xs text-zinc-500 font-mono">
                  DISABLED
                </div>
              {:else if type === "destination" && !status.hasRoute && !status.isSourceChain}
                <div class="text-center text-xs text-zinc-500 font-mono">
                  NO ROUTE
                </div>
              {:else if type === "destination" && !status.hasBucket && status.hasRoute && !status.isSourceChain}
                <div class="text-center text-xs text-zinc-500 font-mono">
                  NOT WHITELISTED
                </div>
              {:else}
                <!-- Available/selectable chains -->
                <div class="text-center text-xs text-zinc-300 font-mono">
                  AVAILABLE
                </div>
              {/if}
            </div>


          </button>
        {/each}
      </div>
    </div>
    
    <!-- Bottom gradient fade -->
    {#if bottomFadeOpacity > 0}
      <div 
        class="absolute bottom-0 left-0 right-0 h-20 bg-gradient-to-t from-zinc-925 to-transparent pointer-events-none z-0" 
        style="opacity: {bottomFadeOpacity}"
        transition:fade={{ duration: 150 }}
      ></div>
    {/if}
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
