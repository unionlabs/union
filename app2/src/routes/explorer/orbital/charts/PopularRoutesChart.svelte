<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { Option, pipe } from "effect"
import type { RouteData } from "../types"

interface Props {
  popularRoutes: Option.Option<RouteData[]>
  popularRoutesTimeScale: Option.Option<Record<string, RouteData[]>>
}

const DEFAULT_ROUTE_DATA: RouteData[] = []

let {
  popularRoutes,
  popularRoutesTimeScale,
}: Props = $props()

// Local item count configuration
const itemCounts = [
  { value: 3, label: "3" },
  { value: 5, label: "5" },
  { value: 7, label: "7" },
  { value: 10, label: "10" },
]

// State management
let selectedTimeScale = $state("5m")
let selectedItemCount = $state(5) // Default to 5 items

// Time scale configuration
const timeScales = [
  { key: "5m", label: "5m" },
  { key: "1h", label: "1h" },
  { key: "1d", label: "1d" },
  { key: "7d", label: "7d" },
  { key: "14d", label: "14d" },
  { key: "30d", label: "30d" },
] as const

// Derived state using Effect Option patterns
const currentData = $derived.by(() => {
  return pipe(
    popularRoutesTimeScale,
    Option.flatMap((timeScaleData) => {
      if (timeScaleData[selectedTimeScale] && timeScaleData[selectedTimeScale].length > 0) {
        return Option.some(timeScaleData[selectedTimeScale])
      }
      return popularRoutes
    }),
    Option.getOrElse(() => DEFAULT_ROUTE_DATA),
    (data) => data.slice(0, selectedItemCount),
  )
})

const hasData = $derived(currentData.length > 0)
const isLoading = $derived(
  !hasData && Option.isNone(popularRoutes),
)

// Get total transfer count for percentage calculation
const totalTransfersForTimeframe = $derived(() => {
  const routeSum = currentData.reduce((sum, route) => sum + route.count, 0)
  return Math.max(routeSum, 1)
})

// Get max count for progress bar visual scaling
const maxCount = $derived(
  currentData.length > 0 ? Math.max(...currentData.map(route => route.count)) : 1,
)

// Utility functions
function formatCount(count: number): string {
  if (count === 0) {
    return "0"
  }
  if (count >= 1000000) {
    return `${(count / 1000000).toFixed(1)}M`
  }
  if (count >= 1000) {
    return `${(count / 1000).toFixed(1)}K`
  }
  return count.toString()
}

function formatChainName(name: string): string {
  return name.toLowerCase().replace(/\s+/g, "_")
}

function formatPercentageChange(change?: number): string {
  if (change === undefined || change === null || !isFinite(change)) {
    return ""
  }
  const sign = change >= 0 ? "+" : ""
  return `(${sign}${change.toFixed(1)}%)`
}

function getPercentageOfTotal(route: RouteData): number {
  return Math.round((route.count / totalTransfersForTimeframe()) * 100)
}
</script>

<Card class="h-full p-0">
  <div class="flex flex-col h-full font-mono">
    <!-- Terminal Header -->
    <header class="flex items-center justify-between p-2 border-b border-zinc-800">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500 text-xs">$</span>
        <h3 class="text-xs text-zinc-300 font-semibold">popular-routes</h3>
        <span class="text-zinc-600 text-xs">--tf={selectedTimeScale}</span>
      </div>
      <div class="text-xs text-zinc-500">
        {#if isLoading}
          loading...
        {:else if !hasData}
          no data yet
        {/if}
      </div>
    </header>

    <!-- Controls -->
    <div class="pt-2 px-2">
      <!-- Mobile: Stack vertically, Desktop: Side by side -->
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-1 mb-1">
        <!-- Time Frame Selector -->
        <div class="flex flex-wrap gap-0.5">
          {#each timeScales as timeScale}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedTimeScale === timeScale.key
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              onclick={() => selectedTimeScale = timeScale.key}
            >
              {timeScale.label}
            </button>
          {/each}
        </div>

        <!-- Item Count Selector -->
        <div class="flex items-center gap-0.5">
          <span class="text-zinc-600 text-xs font-mono">show:</span>
          {#each itemCounts as itemCount}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedItemCount === itemCount.value
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              onclick={() => selectedItemCount = itemCount.value}
            >
              {itemCount.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Routes List -->
    <main class="flex-1 flex flex-col p-2">
      <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
        top_routes:
      </div>

      <div class="flex-1 flex flex-col">
        {#if isLoading}
          <!-- Loading State -->
          <div class="space-y-0.5 flex-1">
            {#each Array(selectedItemCount) as _, index}
              <div class="p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1 text-xs">
                    <Skeleton class="w-2 h-2" />
                    <Skeleton class="w-12 h-2" />
                    <span class="text-zinc-600 text-xs">→</span>
                    <Skeleton class="w-12 h-2" />
                  </div>
                  <div class="flex items-center space-x-1">
                    <Skeleton class="w-8 h-2" />
                  </div>
                </div>
                <div class="flex items-center space-x-2">
                  <div class="flex-1 flex min-w-0">
                    <Skeleton class="w-full h-1" />
                  </div>
                  <Skeleton class="w-6 h-2" />
                </div>
              </div>
            {/each}
          </div>
        {:else if !hasData}
          <!-- No Data State -->
          <div class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <div class="text-zinc-600 font-mono">no_data</div>
            </div>
          </div>
        {:else}
          <!-- Routes Data -->
          <div class="space-y-1 flex-1 overflow-y-auto">
            {#each currentData as route, index}
              <article class="p-2 sm:p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                <!-- Route Header -->
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1 text-xs">
                    <span class="text-zinc-500">#{index + 1}</span>
                    <span class="text-zinc-300 font-medium">
                      {formatChainName(route.fromName)}
                    </span>
                    <span class="text-zinc-600">→</span>
                    <span class="text-zinc-300 font-medium">
                      {formatChainName(route.toName)}
                    </span>
                  </div>
                  <div class="flex items-center space-x-1">
                    {#if route.countChange !== undefined}
                      <span
                        class="text-xs tabular-nums {route.countChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                      >{formatPercentageChange(route.countChange)}</span>
                    {/if}
                    <span class="text-zinc-100 text-xs tabular-nums font-medium">
                      {formatCount(route.count)}
                    </span>
                  </div>
                </div>

                <!-- Progress Bar -->
                <div class="flex items-center space-x-2">
                  <div class="flex-1 flex min-w-0">
                    <!-- Desktop: Thinner bar -->
                    <div class="hidden sm:flex w-full h-1">
                      <div
                        class="bg-zinc-300 h-full transition-all duration-300"
                        style="width: {(route.count / totalTransfersForTimeframe()) * 100}%"
                        title="Count: {route.count}"
                      >
                      </div>
                      <div
                        class="bg-zinc-800 h-full transition-all duration-300"
                        style="width: {100 - (route.count / totalTransfersForTimeframe()) * 100}%"
                      >
                      </div>
                    </div>
                    <!-- Mobile: Thicker bar for better visibility -->
                    <div class="flex sm:hidden w-full h-1.5">
                      <div
                        class="bg-zinc-300 h-full transition-all duration-300"
                        style="width: {(route.count / totalTransfersForTimeframe()) * 100}%"
                        title="Count: {route.count}"
                      >
                      </div>
                      <div
                        class="bg-zinc-800 h-full transition-all duration-300"
                        style="width: {100 - (route.count / totalTransfersForTimeframe()) * 100}%"
                      >
                      </div>
                    </div>
                  </div>
                  <span class="text-zinc-500 text-xs tabular-nums">
                    {getPercentageOfTotal(route)}%
                  </span>
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </div>
    </main>
  </div>
</Card>

<style>
/* Custom scrollbar styling */
.overflow-y-auto::-webkit-scrollbar {
  width: 4px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #27272a;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #52525b;
  border-radius: 2px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #71717a;
}
</style>
