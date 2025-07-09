<script lang="ts">
import SharpChevronDownIcon from "$lib/components/icons/SharpChevronDownIcon.svelte"
import SharpGasIcon from "$lib/components/icons/SharpGasIcon.svelte"
import SharpInfoIcon from "$lib/components/icons/SharpInfoIcon.svelte"
import Anchor from "$lib/components/ui/A.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import * as AppRuntime from "$lib/runtime"
import { FeeStore } from "$lib/stores/fee.svelte"
import { cn } from "$lib/utils"
import { getOptionOrNull, mapOption } from "$lib/utils/snippets.svelte"
import { PriceSource } from "@unionlabs/sdk/PriceOracle"
import { Array as A, BigDecimal as BD, Boolean as B, Option as O, Record as R } from "effect"
import { onDestroy } from "svelte"
import { slide } from "svelte/transition"

let { open, onToggle } = $props()

function toggleExpanded() {
  if (!loading && onToggle) {
    onToggle(!open)
  }
}

const loading: boolean = $derived(B.every([
  FeeStore.isLoading,
]))

const fiber = AppRuntime.runFork$(() => FeeStore.init)

onDestroy(() => {
  fiber.interrupt()
})

const calculating = false
</script>

{#snippet gasButton(props: {
  value: string
  symbol: string
  usd: string
  sources: Record<string, PriceSource>
})}
  <span class="text-xs font-semibold">{props.value} {props.symbol}</span>
  <span class="text-xs text-zinc-500">(${props.usd})</span>
  <Tooltip>
    {#snippet trigger()}
      <div class="flex items-center gap-1 cursor-help group transition-colors">
        <SharpInfoIcon class="size-3.5 text-zinc-500 group-hover:text-zinc-400 transition-colors" />
      </div>
    {/snippet}

    {#snippet content()}
      <div class="flex flex-col gap-4">
        {#each R.toEntries(props.sources) as [k, v]}
          <section>
            <Label>{k}</Label>
            <Anchor href={v.url.toString()}>{v.url}</Anchor>
          </section>
        {/each}
      </div>
    {/snippet}
  </Tooltip>
{/snippet}

{#snippet totalFee(props: {
  totalUsd: string
  value: string
  symbol: string
})}
  <span class="text-zinc-500">(${props.totalUsd})</span>
  <span class="font-semibold">{props.value} {props.symbol}</span>
{/snippet}

{#snippet totalFeeSkeleton()}
  <Skeleton class="h-3 w-12" />
  <Skeleton class="h-3 w-16" />
{/snippet}

{#snippet gasTokenSymbol()}
  {@render getOptionOrNull(FeeStore.symbol)}
{/snippet}

<!-- NOTE: presently only **BOB -> BABYLON** and **BABYLON -> BOB** -->
<div>
  <ul class="text-red-500">
    {#each FeeStore.errors as error}
      <li>{error}</li>
    {/each}
  </ul>
</div>

<div class="w-full overflow-hidden mt-auto">
  <!-- Always visible -->
  <button
    class={cn(
      "w-full p-3 flex items-center justify-between bg-transparent transition-colors text-left h-10",
      open ? "rounded-t-md" : "rounded-md",
      loading ? "cursor-default" : "hover:bg-zinc-800 cursor-pointer",
    )}
    onclick={toggleExpanded}
    disabled={loading}
  >
    <div class="flex items-center gap-1">
      <SharpGasIcon class="size-4 text-zinc-300" />
      {#if !FeeStore.isReady}
        <!-- empty -->
      {:else if loading}
        <Skeleton class="h-3 w-26" />
        <Skeleton class="h-3 w-12" />
      {:else}
        {@render mapOption(
          O.all({
            value: FeeStore.feeDisplay,
            symbol: FeeStore.symbol,
            usd: FeeStore.usdDisplay,
            sources: FeeStore.usdSources,
          }),
          gasButton,
        )}
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
  {#if open && O.isSome(O.all(FeeStore.feeBreakdown))}
    <div
      class="bg-transparent rounded-b-md overflow-hidden border-t border-zinc-800"
      transition:slide={{ duration: 250 }}
    >
      <!-- Fee breakdown -->
      <div class="px-4 pt-3 pb-2 space-y-2">
        {#each A.getSomes(FeeStore.feeBreakdown) as item}
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
                  <div class="flex flex-col text-base">
                    <h2 class="text-white font-bold text-lg">{item.label}</h2>
                    <div class="text-zinc-300 mb-4">{item.description}</div>
                    <div class="bg-zinc-900 p-2 font-mono rounded-sm text-sm">
                      {#each item.steps.calc as [label, op, amount]}
                        <div class="flex items-baseline">
                          <span class="whitespace-nowrap">{@html label}</span>
                          <span
                            aria-hidden="true"
                            class="
                              text-zinc-500
                              flex-1 mx-2
                              h-[0.22em] /* ≈ dot diameter */
                              translate-y-[0.11em] /* push down half its height */
                              bg-[radial-gradient(currentColor_1px,_transparent_1px)]
                              bg-[length:4px_4px]
                              bg-repeat-space
                            "
                          ></span>
                          <span class="whitespace-nowrap text-right">
                            {@html op}&nbsp;{@html amount}
                          </span>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/snippet}
              </Tooltip>
            </div>
            {#if calculating}
              <Skeleton class="h-3 w-20" />
            {:else}
              <span class="text-zinc-400"><code>{item.amount}</code>
                {@render gasTokenSymbol()}</span>
            {/if}
          </div>
        {/each}
      </div>

      <!-- TODO: Receipt transparency -->
      <!--
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
      -->
    </div>
  {/if}
</div>

<style is:global>
:global .batch-savings {
 color: var(--color-green-300) !important;
}
</style>
