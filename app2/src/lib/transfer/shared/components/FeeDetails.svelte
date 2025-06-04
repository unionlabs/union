<script lang="ts">
import SharpChevronDownIcon from "$lib/components/icons/SharpChevronDownIcon.svelte"
import SharpGasIcon from "$lib/components/icons/SharpGasIcon.svelte"
import SharpInfoIcon from "$lib/components/icons/SharpInfoIcon.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import { GasPriceMap } from "$lib/gasprice"
import { GasPrice } from "$lib/gasprice/service"
import { runPromiseExit$, runSync } from "$lib/runtime"
import { cn } from "$lib/utils"
import { PriceOracle } from "@unionlabs/sdk/PriceOracle"
import type { Chain } from "@unionlabs/sdk/schema/chain"
import { Array as A, Cause, Effect, Exit, Option as O, pipe, Predicate, Unify } from "effect"
import { slide } from "svelte/transition"
import { transferData } from "../data/transfer-data.svelte"

let { open, onToggle } = $props()

function toggleExpanded() {
  if (!loading && onToggle) {
    onToggle(!open)
  }
}

const feeConfig = {
  baseFees: { // From graphql
    packetSend: 21000,
    lightClientL1: 150000,
    lightClientL0: 500000,
    packetReceive: 80000,
  },
  gasPrice: 10, // gasPrice from chain
  decimals: 6, // BABY token decimals (in rep)
  feeMultiplier: 0.20, // Union hardcoded fee
  batchDivideNumber: 2, // Api?
  gasTokenDecimals: 6, // Token data
  gasTokenSymbol: "BABY", // Token data
  usdPrice: 0.13, // Gas price from service
}

const applyGasPrice = (gasUnits: number) => gasUnits * feeConfig.gasPrice
const applyFeeMultiplier = (ubbnAmount: number) => ubbnAmount * (1 + feeConfig.feeMultiplier)
const applyBatchDivision = (ubbnAmount: number) => ubbnAmount / feeConfig.batchDivideNumber
const formatToDisplay = (ubbnAmount: number) => {
  const babyAmount = ubbnAmount / Math.pow(10, feeConfig.decimals)
  if (babyAmount < 0.001) {
    return babyAmount.toFixed(6)
  }
  if (babyAmount < 1) {
    return babyAmount.toFixed(4)
  }
  if (babyAmount < 100) {
    return babyAmount.toFixed(3)
  }
  return babyAmount.toFixed(2)
}
const calculateTotalFee = () =>
  pipe(feeConfig.baseFees.packetSend, applyGasPrice, applyFeeMultiplier)
  + pipe(feeConfig.baseFees.lightClientL1, applyGasPrice, applyFeeMultiplier, applyBatchDivision)
  + pipe(feeConfig.baseFees.lightClientL0, applyGasPrice, applyFeeMultiplier, applyBatchDivision)
  + pipe(feeConfig.baseFees.packetReceive, applyGasPrice, applyFeeMultiplier)

const displayFees = $derived({
  packetSend: pipe(
    feeConfig.baseFees.packetSend,
    applyGasPrice,
    applyFeeMultiplier,
    formatToDisplay,
  ),
  lightClientL1: pipe(
    feeConfig.baseFees.lightClientL1,
    applyGasPrice,
    applyFeeMultiplier,
    applyBatchDivision,
    formatToDisplay,
  ),
  lightClientL0: pipe(
    feeConfig.baseFees.lightClientL0,
    applyGasPrice,
    applyFeeMultiplier,
    applyBatchDivision,
    formatToDisplay,
  ),
  packetReceive: pipe(
    feeConfig.baseFees.packetReceive,
    applyGasPrice,
    applyFeeMultiplier,
    formatToDisplay,
  ),
  total: pipe(
    calculateTotalFee(),
    formatToDisplay,
  ),
  totalUsd: pipe(
    calculateTotalFee(),
    formatToDisplay,
    (amount) => parseFloat(amount),
    (amount) => amount * feeConfig.usdPrice,
    (amount) => amount.toFixed(2),
  ),
})

// Fee breakdown items for iteration
const feeBreakdownItems = $derived([
  {
    label: "Packet Send",
    amount: displayFees.packetSend,
    baseFee: feeConfig.baseFees.packetSend,
    isBatched: false,
    description: "Fee for sending the packet to the destination chain",
  },
  {
    label: "Light Client (L1)",
    amount: displayFees.lightClientL1,
    baseFee: feeConfig.baseFees.lightClientL1,
    isBatched: true,
    description: "L1 light client update fee (shared across batch)",
  },
  {
    label: "Light Client (L0)",
    amount: displayFees.lightClientL0,
    baseFee: feeConfig.baseFees.lightClientL0,
    isBatched: true,
    description: "L0 light client update fee (shared across batch)",
  },
  {
    label: "Packet Receive",
    amount: displayFees.packetReceive,
    baseFee: feeConfig.baseFees.packetReceive,
    isBatched: false,
    description: "Fee for receiving the packet on the destination chain",
  },
])

const gasForChain = Effect.fn((chain: Chain) =>
  pipe(
    GasPrice,
    Effect.andThen(({ of }) => of),
    Effect.provide(GasPriceMap.get(chain)),
  )
)

const gasPrices = runPromiseExit$(() =>
  pipe(
    Effect.all({
      source: Effect.transposeMapOption(transferData.sourceChain, gasForChain),
      destination: Effect.transposeMapOption(transferData.destinationChain, gasForChain),
    }, { concurrency: 2 }),
  ), { onInterrupt: "none" })

const usdOfChainGas = Effect.fn((chain: Chain) =>
  pipe(
    PriceOracle,
    Effect.andThen((oracle) => oracle.of(chain.universal_chain_id)),
  )
)

const usdPrices = runPromiseExit$(() =>
  pipe(
    Effect.all({
      source: Effect.transposeMapOption(transferData.sourceChain, usdOfChainGas),
      destination: Effect.transposeMapOption(transferData.destinationChain, usdOfChainGas),
    }, { concurrency: 2 }),
  )
)

const loading = $derived(pipe(
  O.all([gasPrices.current]),
  O.isNone,
))

const calculating = false

const errors = $derived.by(() => {
  // TODO: extract to helper
  const extractError = <E>(x: O.Option<Exit.Exit<any, E>>) =>
    pipe(
      x,
      O.flatMap(Exit.causeOption),
    )
  return pipe(
    [
      extractError(gasPrices.current),
      extractError(usdPrices.current),
    ] as const,
    A.getSomes,
    Unify.unify,
    A.map(Cause.squash),
    A.map(x => (x as any)?.message),
    A.filter(Predicate.isNotUndefined),
  )
})

const gasDisplay = $derived(pipe(
  gasPrices.current,
  // TODO: extract to helper
  O.flatMap(Exit.match({
    onSuccess: O.some,
    onFailure: O.none,
  })),
  O.getOrNull,
))

const usdDisplay = $derived(pipe(
  usdPrices.current,
  // TODO: extract to helper
  O.flatMap(Exit.match({
    onSuccess: O.some,
    onFailure: O.none,
  })),
  O.getOrNull,
))

$effect(() => {
  // console.log("FEES:", transferData.fees)
  // console.log("GAS PRICES:", JSON.stringify(gasPrices.current, null, 2))
})
</script>

<!-- NOTE: presently only **BOB -> BABYLON** and **BABYLON -> BOB** -->
{JSON.stringify(gasPrices.current, null, 2)}
<div>
  <ul class="text-red-500">
    {#each errors as error}
      <li>{error}</li>
    {/each}
  </ul>
  <b>GAS:</b>
  <pre class="w-[350px] overflow-scroll">{JSON.stringify(gasDisplay, null, 2)}</pre>
  <b>USD:</b>
  <pre class="w-[350px] overflow-scroll">{JSON.stringify(usdDisplay, null, 2)}</pre>
</div>
<div class="w-full overflow-hidden mt-auto">
  <!-- Always visible -->
  <button
    class={cn(
      "w-full p-3 flex items-center justify-between bg-zinc-900 transition-colors text-left",
      open ? "rounded-t-md" : "rounded-md",
      loading ? "cursor-default" : "hover:bg-zinc-800 cursor-pointer",
    )}
    onclick={toggleExpanded}
    disabled={loading}
  >
    <div class="flex items-center gap-1">
      <SharpGasIcon class="size-4 text-zinc-300" />
      {#if loading}
        <!-- Show nothing when loading -->
      {:else if calculating}
        <Skeleton class="h-3 w-16" />
        <Skeleton class="h-3 w-12" />
      {:else}
        <span class="text-xs font-semibold">{displayFees.total} {feeConfig.gasTokenSymbol}</span>
        <span class="text-xs text-zinc-500">(${displayFees.totalUsd})</span>
      {/if}
    </div>
    {#if !loading}
      <SharpChevronDownIcon
        class={cn(
          "size-5 text-zinc-400 transition-transform duration-200",
          open && "rotate-180",
        )}
      />
    {/if}
  </button>

  <!-- Expandable content -->
  {#if open}
    <div
      class="bg-zinc-900 rounded-b-md overflow-hidden border-t border-zinc-800"
      transition:slide={{ duration: 250 }}
    >
      <!-- Fee breakdown -->
      <div class="px-4 pt-3 pb-2 space-y-2">
        {#each feeBreakdownItems as item}
          <div class="w-full flex items-center justify-between text-xs">
            <div class="flex items-center gap-1">
              <Tooltip>
                {#snippet trigger()}
                  <div class="flex items-center gap-1 cursor-help group transition-colors">
                    <span class="text-zinc-300 group-hover:text-zinc-200">{item.label}</span>
                    <SharpInfoIcon
                      class="size-3.5 text-zinc-500 group-hover:text-zinc-400 transition-colors"
                    />
                  </div>
                {/snippet}

                {#snippet content()}
                  <div class="text-sm">
                    <div class="font-semibold text-white mb-2">{item.label}</div>
                    <div class="text-zinc-300 mb-4">{item.description}</div>

                    <div class="text-xs font-mono space-y-4">
                      <!-- Input Parameters -->
                      <div>
                        <div class="text-zinc-400 text-xs mb-2 uppercase tracking-wide">
                          Input Parameters
                        </div>
                        <div class="border border-zinc-700 rounded">
                          <div class="grid grid-cols-2 border-b border-zinc-700 bg-zinc-900/50">
                            <div class="px-3 py-2 text-zinc-400 font-medium">Parameter</div>
                            <div
                              class="px-3 py-2 text-zinc-400 font-medium border-l border-zinc-700"
                            >
                              Value
                            </div>
                          </div>
                          <div class="grid grid-cols-2 border-b border-zinc-700">
                            <div class="px-3 py-2 text-zinc-300">Gas cost</div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              {item.baseFee.toLocaleString()}
                            </div>
                          </div>
                          <div class="grid grid-cols-2">
                            <div class="px-3 py-2 text-zinc-300">Gas price</div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              {feeConfig.gasPrice.toLocaleString()} ubbn
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- Calculation Steps -->
                      <div>
                        <div class="text-zinc-400 text-xs mb-2 uppercase tracking-wide">
                          Calculation Steps
                        </div>
                        <div class="border border-zinc-700 rounded">
                          <div class="grid grid-cols-3 border-b border-zinc-700 bg-zinc-900/50">
                            <div class="px-3 py-2 text-zinc-400 font-medium">Step</div>
                            <div
                              class="px-3 py-2 text-zinc-400 font-medium border-l border-zinc-700"
                            >
                              Operation
                            </div>
                            <div
                              class="px-3 py-2 text-zinc-400 font-medium border-l border-zinc-700"
                            >
                              Result
                            </div>
                          </div>

                          <div class="grid grid-cols-3 border-b border-zinc-700">
                            <div class="px-3 py-2 text-zinc-300">Base fee</div>
                            <div class="px-3 py-2 text-zinc-400 border-l border-zinc-700">
                              {item.baseFee.toLocaleString()} × {
                                feeConfig.gasPrice.toLocaleString()
                              }
                            </div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              {
                                (item.baseFee * feeConfig.gasPrice)
                                .toLocaleString()
                              } ubbn
                            </div>
                          </div>

                          <div
                            class="grid grid-cols-3 {item.isBatched ? 'border-b border-zinc-700' : ''}"
                          >
                            <div class="px-3 py-2 text-zinc-300">Protocol fee</div>
                            <div class="px-3 py-2 text-zinc-400 border-l border-zinc-700">
                              + {Math.round(feeConfig.feeMultiplier * 100)}%
                            </div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              +{
                                (item.baseFee * feeConfig.gasPrice
                                * feeConfig.feeMultiplier).toLocaleString()
                              } ubbn
                            </div>
                          </div>

                          {#if item.isBatched}
                            <div class="grid grid-cols-3 border-b border-zinc-700">
                              <div class="px-3 py-2 text-green-300">Batch savings</div>
                              <div class="px-3 py-2 text-green-300 border-l border-zinc-700">
                                ÷ {feeConfig.batchDivideNumber}
                              </div>
                              <div class="px-3 py-2 text-green-400 border-l border-zinc-700">
                                -{
                                  ((item.baseFee * feeConfig.gasPrice
                                  * (1 + feeConfig.feeMultiplier))
                                  - (item.baseFee * feeConfig.gasPrice
                                      * (1 + feeConfig.feeMultiplier))
                                    / feeConfig.batchDivideNumber)
                                  .toLocaleString()
                                } ubbn
                              </div>
                            </div>
                          {/if}

                          <div class="grid grid-cols-3 bg-zinc-800/50 border-t border-zinc-700">
                            <div class="px-3 py-2 text-white font-semibold">Total</div>
                            <div
                              class="px-3 py-2 text-zinc-300 border-l border-zinc-700 font-medium"
                            >
                            </div>
                            <div
                              class="px-3 py-2 text-white font-semibold border-l border-zinc-700"
                            >
                              {#if item.isBatched}
                                {
                                  ((item.baseFee * feeConfig.gasPrice
                                  * (1 + feeConfig.feeMultiplier))
                                  / feeConfig.batchDivideNumber)
                                  .toLocaleString()
                                } ubbn
                              {:else}
                                {
                                  (item.baseFee * feeConfig.gasPrice
                                  * (1 + feeConfig.feeMultiplier))
                                  .toLocaleString()
                                } ubbn
                              {/if}
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- Unit Conversion -->
                      <div>
                        <div class="text-zinc-400 text-xs mb-2 uppercase tracking-wide">
                          Unit Conversion
                        </div>
                        <div class="border border-zinc-700 rounded">
                          <div class="grid grid-cols-3 border-b border-zinc-700 bg-zinc-900/50">
                            <div class="px-3 py-2 text-zinc-400 font-medium">Amount (ubbn)</div>
                            <div
                              class="px-3 py-2 text-zinc-400 font-medium border-l border-zinc-700"
                            >
                              Operation
                            </div>
                            <div
                              class="px-3 py-2 text-zinc-400 font-medium border-l border-zinc-700"
                            >
                              Result
                            </div>
                          </div>
                          <div class="grid grid-cols-3">
                            <div class="px-3 py-2 text-zinc-300">
                              {#if item.isBatched}
                                {
                                  ((item.baseFee * feeConfig.gasPrice
                                  * (1 + feeConfig.feeMultiplier))
                                  / feeConfig.batchDivideNumber)
                                  .toLocaleString()
                                }
                              {:else}
                                {
                                  (item.baseFee * feeConfig.gasPrice
                                  * (1 + feeConfig.feeMultiplier))
                                  .toLocaleString()
                                }
                              {/if}
                            </div>
                            <div class="px-3 py-2 text-zinc-400 border-l border-zinc-700">
                              ÷ 10^{feeConfig.decimals}
                            </div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              {item.amount} {feeConfig.gasTokenSymbol}
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- Final Amount with USD -->
                      <div>
                        <div class="text-zinc-400 text-xs mb-1.5 uppercase tracking-wide">
                          Final Amount
                        </div>
                        <div class="border-2 border-zinc-600 rounded-lg bg-zinc-900/30">
                          <div class="px-3 py-2 text-center">
                            <div class="text-sm font-bold text-white mb-0.5">
                              {item.amount} {feeConfig.gasTokenSymbol}
                            </div>
                            <div class="text-xs text-zinc-400">
                              ≈ ${
                                (parseFloat(item.amount) * feeConfig.usdPrice)
                                .toFixed(4)
                              } USD
                            </div>
                            <div class="text-xs text-zinc-500">
                              @ ${feeConfig.usdPrice} per {feeConfig.gasTokenSymbol}
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                {/snippet}
              </Tooltip>
            </div>
            {#if calculating}
              <Skeleton class="h-3 w-20" />
            {:else}
              <span class="text-zinc-400">{item.amount} {feeConfig.gasTokenSymbol}</span>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Total fee -->
      <div class="border-t border-zinc-800 px-4 py-3">
        <div class="flex items-center justify-between text-xs">
          <span class="text-white font-semibold">Total</span>
          <div class="flex items-center gap-2">
            {#if calculating}
              <Skeleton class="h-3 w-12" />
              <Skeleton class="h-3 w-16" />
            {:else}
              <span class="text-zinc-500">(${displayFees.totalUsd})</span>
              <span class="font-semibold">{displayFees.total} {feeConfig.gasTokenSymbol}</span>
            {/if}
          </div>
        </div>
      </div>

      <!-- Route visualization -->
      <div class="border-t border-zinc-800 px-4 py-3">
        <div class="flex items-center justify-between text-xs mb-3">
          <span class="text-zinc-400 font-medium">Route</span>
          <span class="text-zinc-400">ETA 4m</span>
        </div>
        <div class="flex items-center gap-3">
          <div class="size-5 bg-blue-500 rounded-full flex items-center justify-center">
            <span class="text-xs font-bold text-white">Ξ</span>
          </div>
          <div class="flex-1 h-px bg-zinc-600"></div>
          <div class="size-5 bg-blue-400 rounded-full flex items-center justify-center">
            <span class="text-xs font-bold text-white">$</span>
          </div>
          <div class="flex-1 h-px bg-zinc-600"></div>
          <div class="size-5 bg-orange-500 rounded-full flex items-center justify-center">
            <span class="text-xs font-bold text-white">₿</span>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
