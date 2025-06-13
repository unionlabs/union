<script lang="ts">
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"
import Card from "./ui/Card.svelte"
import Skeleton from "./ui/Skeleton.svelte"

let { 
  popularRoutes = [],
  popularRoutesTimeScale = {}
}: { 
  popularRoutes?: Array<{
    route: string
    count: number
    fromChain: string
    toChain: string
    fromName: string
    toName: string
  }>
  popularRoutesTimeScale?: Record<string, Array<{
    route: string
    count: number
    fromChain: string
    toChain: string
    fromName: string
    toName: string
  }>>
} = $props()

// Time scale selection
let selectedTimeScale: string = $state('1m')
const timeScales = ['1m', '1h', '1d', '7d', '14d', '30d']

// Get current data based on selected time scale
const currentData = $derived(
  (() => {
    let data = []
    if (popularRoutesTimeScale && popularRoutesTimeScale[selectedTimeScale] && popularRoutesTimeScale[selectedTimeScale].length > 0) {
      data = popularRoutesTimeScale[selectedTimeScale]
    } else {
      data = popularRoutes
    }
    
    // Limit to top 5
    return data?.slice(0, 5) || []
  })()
)

// Use backend data directly
const maxCount = $derived(
  currentData.length > 0 ? currentData[0].count : 1
)

// Monochrome zinc theme colors for different routes
const ROUTE_COLORS = [
  "#f4f4f5", // zinc-100 - lightest
  "#e4e4e7", // zinc-300 
  "#d4d4d8", // zinc-300
  "#a1a1aa", // zinc-400
  "#71717a", // zinc-500 - darkest
]
</script>

<Card class="h-full">
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <h3 class="text-sm font-medium text-zinc-300">Popular Routes</h3>
        <span class="text-xs text-zinc-500">Top {currentData.length}</span>
      </div>
      
      <!-- Time scale selector -->
      <div class="flex items-center gap-1 text-xs">
        {#each timeScales as timeScale}
          <button
            onclick={() => selectedTimeScale = timeScale}
            class="px-1.5 py-0.5 rounded transition-colors {
              selectedTimeScale === timeScale 
                ? 'bg-zinc-700 text-zinc-100' 
                : 'text-zinc-500 hover:text-zinc-300'
            }"
          >
            {timeScale}
          </button>
        {/each}
      </div>
    </div>
    
    <div class="flex-1 space-y-2 min-h-0 overflow-y-auto">
      {#each currentData as route, index}
        <div class="group">
          <!-- Route info -->
          <div class="flex items-center justify-between mb-1">
            <div class="flex items-center gap-2 text-xs">
              <div 
                class="w-2 h-2 rounded-full flex-shrink-0"
                style="background-color: {ROUTE_COLORS[index % ROUTE_COLORS.length]}"
              ></div>
              <span class="text-zinc-300 truncate">
                {route.fromName}
              </span>
              <span class="text-zinc-500">→</span>
              <span class="text-zinc-300 truncate">
                {route.toName}
              </span>
            </div>
            <span class="text-zinc-400 text-xs flex-shrink-0 ml-2">
              {route.count}
            </span>
          </div>
          
          <!-- Progress bar -->
          <div class="w-full bg-zinc-800 rounded-full h-1.5 overflow-hidden">
            <div 
              class="h-full rounded-full transition-all duration-500 ease-out"
              style="
                background-color: {ROUTE_COLORS[index % ROUTE_COLORS.length]};
                width: {(route.count / maxCount) * 100}%;
                transition-delay: {index * 50}ms;
              "
            ></div>
          </div>
        </div>
      {/each}
      
      {#if currentData.length === 0}
        <!-- Skeleton loading state -->
        {#each Array(5) as _, index}
          <div class="group">
            <!-- Route info skeleton -->
            <div class="flex items-center justify-between mb-1">
              <div class="flex items-center gap-2 text-xs">
                <Skeleton class="w-2 h-2 rounded-full" />
                <Skeleton class="h-3 w-16" />
                <Skeleton class="h-3 w-3" />
                <Skeleton class="h-3 w-20" />
              </div>
              <Skeleton class="h-3 w-6" />
            </div>
            
            <!-- Progress bar skeleton -->
            <Skeleton class="w-full h-1.5 rounded-full" />
          </div>
        {/each}
      {/if}
    </div>
  </div>
</Card> 