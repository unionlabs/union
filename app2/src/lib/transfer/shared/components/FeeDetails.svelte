<script lang="ts">
import SharpChevronDownIcon from "$lib/components/icons/SharpChevronDownIcon.svelte"
import SharpGasIcon from "$lib/components/icons/SharpGasIcon.svelte"
import SharpInfoIcon from "$lib/components/icons/SharpInfoIcon.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import { runPromiseExit$, runSync } from "$lib/runtime"
import { FeeStore } from "$lib/stores/fee.svelte"
import { cn } from "$lib/utils"
import { PriceOracle } from "@unionlabs/sdk/PriceOracle"
import type { Chain } from "@unionlabs/sdk/schema/chain"
import type { Fees } from "@unionlabs/sdk/schema/fee"
import {
  Array as A,
  BigDecimal as BD,
  BigInt as BI,
  Cause,
  Effect,
  Either as E,
  Exit,
  Match,
  Option as O,
  pipe,
  Predicate,
  Record as R,
  Struct,
  Unify,
} from "effect"
import { constant, flow } from "effect/Function"
import { slide } from "svelte/transition"
import { transferData } from "../data/transfer-data.svelte"

let { open, onToggle } = $props()

function toggleExpanded() {
  if (!loading && onToggle) {
    onToggle(!open)
  }
}

const displayTotals = $derived({
  total: pipe(
    O.map(feeConfig, (x) => x.formatToDisplay),
    O.flatMap((format) => O.map(totalFee, format)),
  ),
  totalUsd: pipe(
    "$123.456",
    // calculateTotalFee(),
    // formatToDisplay,
    // (amount) => parseFloat(amount),
    // (amount) => amount * feeConfig.usdPrice,
    // (amount) => amount.toFixed(2),
  ),
})

const loading = $derived(pipe(
  O.all([FeeStore.gasPrices.current]),
  O.isNone,
))

const calculating = false
</script>

{#snippet BigDecimal(x: BD.BigDecimal)}
  {BD.format(x)}
{/snippet}

<!-- NOTE: presently only **BOB -> BABYLON** and **BABYLON -> BOB** -->
<div>
  <ul class="text-red-500">
    {#each FeeStore.errors as error}
      <li>{error}</li>
    {/each}
  </ul>
  <!--
  <b>GAS:</b>
  <pre class="w-[350px] overflow-scroll">{JSON.stringify(gasDisplay, null, 2)}</pre>
  <b>USD:</b>
  <pre class="w-[350px] overflow-scroll">{JSON.stringify(usdDisplay, null, 2)}</pre>
  -->
  <!--
  {FeeStore.toasts}
  {FeeStore.a.current}
  -->
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
        {@const _feeConfig = feeConfig.value}
        <span class="text-xs font-semibold">{O.getOrUndefined(totalFee)?.value}
          {_feeConfig.gasTokenSymbol}</span>
        <span class="text-xs text-zinc-500">(${displayTotals.totalUsd})</span>
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
  {#if open && O.isSome(feeConfig) && O.isSome(displayFees)}
    {@const _feeConfig = feeConfig.value}
    {@const _displayFees = displayFees.value}
    <div
      class="bg-zinc-900 rounded-b-md overflow-hidden border-t border-zinc-800"
      transition:slide={{ duration: 250 }}
    >
      <!-- Fee breakdown -->
      <div class="px-4 pt-3 pb-2 space-y-2">
        {#each A.getSomes(feeBreakdownItems) as item}
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
                              {@render BigDecimal(item.baseFee)}
                            </div>
                          </div>
                          <div class="grid grid-cols-2">
                            <div class="px-3 py-2 text-zinc-300">Gas price</div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              {@render BigDecimal(_feeConfig.gasPrice)} ubbn
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
                              {@render BigDecimal(item.baseFee)} ×
                              {@render BigDecimal(_feeConfig.gasPrice)}
                            </div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              {@render BigDecimal(
                            BD.multiply(
                              item.baseFee,
                              _feeConfig.gasPrice,
                            ),
                          )}
                              ubbn
                            </div>
                          </div>

                          <div
                            class="grid grid-cols-3 {item.isBatched ? 'border-b border-zinc-700' : ''}"
                          >
                            <div class="px-3 py-2 text-zinc-300">Protocol fee</div>
                            <div class="px-3 py-2 text-zinc-400 border-l border-zinc-700">
                              +
                              {@render BigDecimal(
                            BD.multiply(
                              _feeConfig.feeMultiplier,
                              BD.unsafeFromNumber(10),
                            ),
                          )}%
                            </div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              +{
                                (BD.multiply(
                                  BD.multiply(
                                    item.baseFee,
                                    _feeConfig.gasPrice,
                                  ),
                                  _feeConfig.feeMultiplier,
                                )).toLocaleString()
                              } ubbn
                            </div>
                          </div>

                          {#if item.isBatched}
                            <div class="grid grid-cols-3 border-b border-zinc-700">
                              <div class="px-3 py-2 text-green-300">Batch savings</div>
                              <div class="px-3 py-2 text-green-300 border-l border-zinc-700">
                                ÷ {_feeConfig.batchDivideNumber}
                              </div>
                              <div class="px-3 py-2 text-green-400 border-l border-zinc-700">
                                <!--
                                -{
                                  ((item.baseFee * _feeConfig.gasPrice
                                  * (1 + feeConfig.feeMultiplier))
                                  - (item.baseFee * feeConfig.gasPrice
                                      * (1 + feeConfig.feeMultiplier))
                                    / feeConfig.batchDivideNumber)
                                  .toLocaleString()
                                } ubbn
                                -->
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
                              <!--
                              {#if item.isBatched}
                                {
                                  ((item.baseFee * _feeConfig.gasPrice
                                  * (1 + _feeConfig.feeMultiplier))
                                  / _feeConfig.batchDivideNumber)
                                  .toLocaleString()
                                } ubbn
                              {:else}
                                {
                                  (item.baseFee * _feeConfig.gasPrice
                                  * (1 + _feeConfig.feeMultiplier))
                                  .toLocaleString()
                                } ubbn
                              {/if}
                              -->
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
                              <!--
                              {#if item.isBatched}
                                {
                                  ((item.baseFee * _feeConfig.gasPrice
                                  * (1 + _feeConfig.feeMultiplier))
                                  / _feeConfig.batchDivideNumber)
                                  .toLocaleString()
                                }
                              {:else}
                                {
                                  (item.baseFee * _feeConfig.gasPrice
                                  * (1 + _feeConfig.feeMultiplier))
                                  .toLocaleString()
                                }
                              {/if}
                              -->
                            </div>
                            <div class="px-3 py-2 text-zinc-400 border-l border-zinc-700">
                              ÷ 10^{_feeConfig.decimals}
                            </div>
                            <div class="px-3 py-2 text-white border-l border-zinc-700">
                              {item.amount} {_feeConfig.gasTokenSymbol}
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
                              {item.amount} {_feeConfig.gasTokenSymbol}
                            </div>
                            <div class="text-xs text-zinc-400">
                              ≈ ${
                                (parseFloat(item.amount) * _feeConfig.usdPrice)
                                .toFixed(4)
                              } USD
                            </div>
                            <div class="text-xs text-zinc-500">
                              @ ${_feeConfig.usdPrice} per {_feeConfig.gasTokenSymbol}
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
              <span class="text-zinc-400">{item.amount} {_feeConfig.gasTokenSymbol}</span>
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
              <span class="text-zinc-500">(${displayTotals.totalUsd})</span>
              <span class="font-semibold">{O.getOrUndefined(displayTotals.total)}
                {O.getOrUndefined(feeConfig)?.gasTokenSymbol}</span>
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
