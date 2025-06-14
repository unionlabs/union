<script lang="ts">
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { chains } from "$lib/stores/chains.svelte"
import { selectedItemCount, itemCounts } from "$lib/stores/chartSettings"
import { Option } from "effect"
import Card from "./ui/Card.svelte"
import Skeleton from "./ui/Skeleton.svelte"

let { 
  popularRoutes = [],
  popularRoutesTimeScale = {},
  dataAvailability = {
    hasMinute: false,
    hasHour: false,
    hasDay: false,
    has7Days: false,
    has14Days: false,
    has30Days: false
  }
}: { 
  popularRoutes?: Array<{
    route: string
    count: number
    fromChain: string
    toChain: string
    fromName: string
    toName: string
    countChange?: number
  }>
  popularRoutesTimeScale?: Record<string, Array<{
    route: string
    count: number
    fromChain: string
    toChain: string
    fromName: string
    toName: string
    countChange?: number
  }>>
  dataAvailability?: {
    hasMinute: boolean
    hasHour: boolean
    hasDay: boolean
    has7Days: boolean
    has14Days: boolean
    has30Days: boolean
  }
} = $props()

// Time scale selection
let selectedTimeScale: string = $state('1m')
const timeScales = [
  { key: '1m', label: '1m' },
  { key: '1h', label: '1h' },
  { key: '1d', label: '1d' },
  { key: '7d', label: '7d' },
  { key: '14d', label: '14d' },
  { key: '30d', label: '30d' }
]

// Use shared item count selection

// Get current data based on selected time scale
const currentData = $derived(
  (() => {
    let data = []
    if (popularRoutesTimeScale && popularRoutesTimeScale[selectedTimeScale] && popularRoutesTimeScale[selectedTimeScale].length > 0) {
      data = popularRoutesTimeScale[selectedTimeScale]
    } else {
      data = popularRoutes
    }
    
    // Limit to selected number for display
    return data?.slice(0, $selectedItemCount) || []
  })()
)

// Get total transfer count for percentage calculation (fallback to route counts)
const totalTransfersForTimeframe = $derived(() => {
  // Use sum of route counts as the total for percentage calculation
  const routeSum = currentData.reduce((sum, route) => sum + route.count, 0)
  return Math.max(routeSum, 1)
})

// Get max count for progress bar scaling (visual scaling)
const maxCount = $derived(
  currentData.length > 0 ? Math.max(...currentData.map(route => route.count)) : 1
)

// Terminal-style count formatting
function formatCount(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K`
  return count.toString().padStart(3, ' ')
}

// Terminal-style chain name formatting
function formatChainName(name: string): string {
  return name.toLowerCase().replace(/\s+/g, '_')
}

// Helper function to format percentage changes
function formatPercentageChange(change?: number): string {
  if (change === undefined || change === null || !isFinite(change)) return ''
  const sign = change >= 0 ? '+' : ''
  return `(${sign}${change.toFixed(1)}%)`
}

// Check if time frame data is available using global dataAvailability
function isTimeFrameAvailable(timeFrameKey: string): boolean {
  const availabilityMap: Record<string, keyof typeof dataAvailability> = {
    '1m': 'hasMinute',
    '1h': 'hasHour', 
    '1d': 'hasDay',
    '7d': 'has7Days',
    '14d': 'has14Days',
    '30d': 'has30Days'
  }
  
  return dataAvailability[availabilityMap[timeFrameKey]] || false
}

// Get the first available timeframe (prioritizing shorter timeframes)
function getFirstAvailableTimeframe(): string {
  for (const timeScale of timeScales) {
    if (isTimeFrameAvailable(timeScale.key)) {
      return timeScale.key
    }
  }
  return '1m' // Fallback to 1m even if not available yet
}

// Auto-update selected timeframe when data becomes available
$effect(() => {
  const firstAvailable = getFirstAvailableTimeframe()
  
  // If current selection is not available, switch to first available
  if (!isTimeFrameAvailable(selectedTimeScale)) {
    selectedTimeScale = firstAvailable
  }
})

// Check if we have data for the selected timeframe
const hasData = $derived(currentData.length > 0)
const isLoading = $derived(!hasData && popularRoutes.length === 0)
</script>

<Card class="h-full p-0">
  <div class="flex flex-col h-full font-mono">
    <!-- Terminal Header -->
    <div class="flex items-center justify-between p-3 border-b border-zinc-800">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500">$</span>
        <h3 class="text-xs text-zinc-300">popular-routes</h3>
        <span class="text-zinc-600 text-xs">--tf={selectedTimeScale}</span>
      </div>
      <div class="flex items-center space-x-1">
        {#if isLoading}
          <span class="text-zinc-600 text-xs animate-pulse">●</span>
        {:else}
          <span class="text-green-500 text-xs">●</span>
        {/if}
        <span class="text-xs text-zinc-500">live</span>
      </div>
    </div>

    <!-- Time Frame Selector - Terminal Style -->
    <div class="pt-3 px-3">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="flex flex-wrap gap-1">
          {#each timeScales as timeScale}
            <button
              class="px-1.5 py-0.5 text-xs font-mono border transition-colors {
                selectedTimeScale === timeScale.key 
                  ? 'border-zinc-500 bg-zinc-800 text-zinc-200' 
                  : isTimeFrameAvailable(timeScale.key)
                    ? 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                    : 'border-zinc-800 bg-zinc-950 text-zinc-600 cursor-not-allowed'
              }"
              disabled={!isTimeFrameAvailable(timeScale.key)}
              onclick={() => selectedTimeScale = timeScale.key}
            >
              {timeScale.label}
            </button>
          {/each}
        </div>
        
        <!-- Item Count Selector -->
        <div class="flex items-center gap-1">
          <span class="text-zinc-600 text-xs font-mono">show:</span>
          {#each itemCounts as itemCount}
            <button
              class="px-1.5 py-0.5 text-xs font-mono border transition-colors {
                $selectedItemCount === itemCount.value
                  ? 'border-zinc-500 bg-zinc-800 text-zinc-200' 
                  : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
              }"
              onclick={() => selectedItemCount.set(itemCount.value)}
            >
              {itemCount.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Routes List - Terminal Style -->
    <div class="flex-1 flex flex-col p-3">
      <div class="text-xs text-zinc-500 font-mono mb-1">
        top_routes:
      </div>
      
      <div class="flex-1 flex flex-col">
        {#if isLoading}
          <!-- Loading skeletons -->
          <div class="space-y-0.5 flex-1">
            {#each Array($selectedItemCount) as _, index}
              <div class="p-2 bg-zinc-900 border border-zinc-800 rounded">
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1">
                    <Skeleton class="w-3 h-3" />
                    <Skeleton class="w-16 h-3" />
                    <span class="text-zinc-600 text-xs">→</span>
                    <Skeleton class="w-16 h-3" />
                  </div>
                  <Skeleton class="w-8 h-3" />
                </div>
                <div class="flex items-center space-x-1">
                  <span class="text-zinc-600 text-xs">[</span>
                  <Skeleton class="flex-1 h-2" />
                  <span class="text-zinc-600 text-xs">]</span>
                  <Skeleton class="w-8 h-3" />
                </div>
              </div>
            {/each}
          </div>
        {:else if !hasData}
          <!-- No data state -->
          <div class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <div class="text-zinc-600 font-mono">no_data</div>
            </div>
          </div>
        {:else}
          <!-- Routes data -->
          <div class="space-y-0.5 flex-1">
            {#each currentData as route, index}
              <div class="p-2 bg-zinc-900 border border-zinc-800 rounded">
                <!-- Route header -->
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1 text-xs">
                    <span class="text-zinc-500 w-3">{(index + 1)}.</span>
                    <span class="text-zinc-300">
                      {formatChainName(route.fromName)}
                    </span>
                    <span class="text-zinc-600">→</span>
                    <span class="text-zinc-300">
                      {formatChainName(route.toName)}
                    </span>
                  </div>
                  <span class="text-zinc-100 text-xs tabular-nums">
                    {#if isTimeFrameAvailable(selectedTimeScale) && route.countChange !== undefined}
                      <span class="text-xs mr-1 {route.countChange && route.countChange >= 0 ? 'text-green-400' : 'text-red-400'}">{formatPercentageChange(route.countChange)}</span>
                    {/if}
                    {formatCount(route.count)}
                  </span>
                </div>
                
                <!-- Progress bar - Terminal style -->
                <div class="flex items-center space-x-1">
                  <span class="text-zinc-600 text-xs">[</span>
                  <div class="flex-1 flex">
                    {#each Array(50) as _, i}
                      <span class="flex-1 text-center text-[7px] leading-none {
                        i < Math.floor((route.count / totalTransfersForTimeframe()) * 50) 
                          ? 'text-zinc-400' 
                          : 'text-zinc-800'
                      }">█</span>
                    {/each}
                  </div>
                  <span class="text-zinc-600 text-[10px]">]</span>
                  <span class="text-zinc-500 text-[10px] tabular-nums">
                    {Math.round((route.count / totalTransfersForTimeframe()) * 100)}%
                  </span>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>


  </div>
</Card> 