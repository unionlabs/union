<script lang="ts">
import MultiSwitch from "$lib/components/ui/MultiSwitch.svelte"
import { chainLogoMap } from "$lib/constants/chain-logos"
import { DISABLED_CHAINS } from "$lib/constants/disabled-chains"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { signingMode } from "$lib/transfer/signingMode.svelte"
import { cn } from "$lib/utils"
import type { Chain, Token, TokenWrapping } from "@unionlabs/sdk/schema"
import { Match, Option, pipe, Tuple } from "effect"
import * as A from "effect/Array"
import { constant, constVoid, identity } from "effect/Function"
import { fade } from "svelte/transition"

type Props = {
  type: "source" | "destination"
  onSelect: () => void
}

const { type, onSelect }: Props = $props()

type ChainWithAvailability = ReturnType<typeof Tuple.make<[Chain, boolean]>>

let topFadeOpacity = $state(0)
let bottomFadeOpacity = $state(1)
let hasAnimated = $state(false)

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

const filterByNet = (chains: Array<Chain>) =>
  pipe(
    Match.value(transferData.net),
    Match.when("all", constant(chains)),
    Match.orElse((net) => A.filter(chains, x => x.testnet === (net === "testnet"))),
  )

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
        filterByNet,
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

// Reset animation state when selector type changes
$effect(() => {
  void type // Track the type prop
  hasAnimated = false
})

// Set hasAnimated to true after first render with chains
$effect(() => {
  if (Option.isSome(filteredChains) && !hasAnimated) {
    setTimeout(() => {
      hasAnimated = true
    }, 1000) // Allow animations to complete
  }
})
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
        >
        </div>
      {/if}

      <div
        class="flex flex-col gap-1 p-2 pb-5 max-h-[459px] overflow-y-auto scrollbar-thin scrollbar-track-zinc-900 scrollbar-thumb-zinc-700"
        onscroll={handleScroll}
      >
        {#each chainss as
          chainWithAvailability,
          index
          (chainWithAvailability[0].universal_chain_id || index)
        }
          {@const [chain, hasBucket] = chainWithAvailability}
          {@const status = getChainStatus(chain, hasBucket)}
          {@const chainLogo = chain.universal_chain_id
          ? chainLogoMap.get(chain.universal_chain_id)
          : null}

          <button
            style={!hasAnimated ? `animation-delay: ${index * 30}ms;` : ""}
            class={cn(
              "group relative flex items-center gap-3 w-full px-4 py-3 rounded border",
              "transition-colors duration-100", // Only animate colors, not opacity/transform
              !hasAnimated && "animate-fade-in-up",
              status.isSelected
                ? "bg-zinc-900 border-accent text-white"
                : status.isDisabled
                ? "bg-zinc-900/35 border-zinc-800/40 cursor-not-allowed"
                : "bg-zinc-900 border-zinc-800 hover:border-zinc-600 cursor-pointer",
            )}
            onclick={() => !status.isDisabled && updateSelectedChain(chain)}
            disabled={status.isDisabled}
          >
            <!-- Chain content (gets dimmed when disabled) -->
            <div
              class={cn(
                "flex items-center gap-3 flex-1 min-w-0 transition-all duration-100",
                status.isDisabled && "opacity-40",
              )}
            >
              <!-- Chain logo -->
              <div
                class={cn(
                  "relative w-10 h-10 flex items-center justify-center rounded-full transition-all duration-100 flex-shrink-0",
                  "bg-zinc-800 border border-zinc-700",
                )}
              >
                {#if chainLogo?.color}
                  <img
                    src={chainLogo.color}
                    alt={chain.display_name}
                    class={cn(
                      "w-6 h-6 object-contain",
                      status.isDisabled && "grayscale",
                    )}
                  />
                {:else}
                  <!-- Fallback icon for chains without logos -->
                  <div
                    class={cn(
                      "w-4 h-4 bg-gradient-to-br from-accent/60 to-accent/80 rounded-full",
                      status.isDisabled && "grayscale",
                    )}
                  >
                  </div>
                {/if}
              </div>

              <!-- Chain name -->
              <span
                class={cn(
                  "text-sm font-medium truncate",
                  status.isSelected ? "text-white" : "text-zinc-200",
                  status.isDisabled && "text-zinc-400",
                )}
              >
                {chain.display_name}
              </span>
            </div>

            <!-- Status label -->
            <div class="text-right flex-shrink-0 ml-3">
              {#if status.isSourceChain}
                <div class="flex items-center gap-1 bg-accent/10 border border-accent/20 px-1.5 py-0.5 rounded text-accent">
                  <svg
                    class="w-2.5 h-2.5"
                    fill="currentColor"
                    viewBox="0 0 16 16"
                  >
                    <circle
                      cx="8"
                      cy="8"
                      r="3"
                    />
                  </svg>
                  <span class="text-xs font-mono leading-none">SOURCE</span>
                </div>
              {:else if chain.universal_chain_id
              && DISABLED_CHAINS.includes(chain.universal_chain_id)}
                <div class="flex items-center gap-1 bg-red-500/10 border border-red-500/20 px-1.5 py-0.5 rounded text-red-400">
                  <svg
                    class="w-2.5 h-2.5"
                    fill="currentColor"
                    viewBox="0 0 16 16"
                  >
                    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z" />
                    <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z" />
                  </svg>
                  <span class="text-xs font-mono leading-none">DISABLED</span>
                </div>
              {:else if type === "destination" && !status.hasRoute && !status.isSourceChain}
                <div class="flex items-center gap-1 bg-orange-500/10 border border-orange-500/20 px-1.5 py-0.5 rounded text-orange-400">
                  <svg
                    class="w-2.5 h-2.5"
                    fill="currentColor"
                    viewBox="0 0 16 16"
                  >
                    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z" />
                    <path d="M7.002 11a1 1 0 1 1 2 0 1 1 0 0 1-2 0zM7.1 4.995a.905.905 0 1 1 1.8 0l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 4.995z" />
                  </svg>
                  <span class="text-xs font-mono leading-none">NO ROUTE</span>
                </div>
              {:else if type === "destination" && !status.hasBucket && status.hasRoute
              && !status.isSourceChain}
                <div class="flex items-center gap-1 bg-yellow-500/10 border border-yellow-500/20 px-1.5 py-0.5 rounded text-yellow-400">
                  <svg
                    class="w-2.5 h-2.5"
                    fill="currentColor"
                    viewBox="0 0 16 16"
                  >
                    <path d="M8 1a2.5 2.5 0 0 1 2.5 2.5V4h-5v-.5A2.5 2.5 0 0 1 8 1zm3.5 3v-.5a3.5 3.5 0 1 0-7 0V4H1v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V4h-3.5zM2 5h12v9a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V5z" />
                  </svg>
                  <span class="text-xs font-mono leading-none">LOCKED</span>
                </div>
              {/if}
            </div>
          </button>
        {/each}

        {#if type === "source"}
          <div class="w-full text-center text-sm text-zinc-400 italic pt-8 pb-2">
            Looking for more?
          </div>
          <MultiSwitch
            selectedKey={transferData.net}
            orientation="horizontal"
            class="w-full"
            onChange={(change) => {
              transferData.net = change.key
            }}
            options={[
              {
                key: "mainnet",
                label: "MAINNET",
              },
              {
                key: "all",
                label: "ALL",
              },
              {
                key: "testnet",
                label: "TESTNET",
              },
            ]}
          />
        {/if}
      </div>
    </div>

    <!-- Bottom gradient fade -->
    {#if bottomFadeOpacity > 0}
      <div
        class="absolute bottom-0 left-0 right-0 h-20 bg-gradient-to-t from-zinc-925 to-transparent pointer-events-none z-0"
        style="opacity: {bottomFadeOpacity}"
        transition:fade={{ duration: 150 }}
      >
      </div>
    {/if}
  {:else}
    <div class="py-8 text-center">
      <div class="inline-flex items-center gap-3 px-4 py-3 bg-zinc-900/50 rounded-xl border border-zinc-700/50">
        <div class="w-5 h-5 border-2 border-accent border-t-transparent rounded-full animate-spin">
        </div>
        <span class="text-zinc-400 font-medium">Loading chains...</span>
      </div>
    </div>
  {/if}
</div>

<style>
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translate3d(0, 20px, 0) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translate3d(0, 0, 0) scale(1);
  }
}

.animate-fade-in-up {
  animation: fade-in-up 0.4s ease-out forwards;
  will-change: transform, opacity;
  opacity: 0;
}
</style>
