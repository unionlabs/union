<script lang="ts">
import Card from "./ui/Card.svelte"
import Skeleton from "./ui/Skeleton.svelte"

let { 
  activeSendersTimeScale = {},
  activeReceiversTimeScale = {}
}: { 
  activeSendersTimeScale?: Record<string, Array<{
    address: string
    count: number
    displayAddress: string
  }>>
  activeReceiversTimeScale?: Record<string, Array<{
    address: string
    count: number
    displayAddress: string
  }>>
} = $props()

// Toggle between senders and receivers
let viewMode: 'senders' | 'receivers' = $state('senders')

// Time scale selection
let selectedTimeScale: string = $state('1m')
const timeScales = ['1m', '1h', '1d', '7d', '14d', '30d']

// Get current data based on view mode and time scale
const currentData = $derived(
  (() => {
    const timeScaleData = viewMode === 'senders' ? activeSendersTimeScale : activeReceiversTimeScale
    
    // Use time-scale data only
    let data: Array<{
      address: string
      count: number
      displayAddress: string
    }> = []
    if (timeScaleData && timeScaleData[selectedTimeScale]) {
      data = timeScaleData[selectedTimeScale]
    }
    
    // Limit to top 5
    return data?.slice(0, 5) || []
  })()
)

const maxCount = $derived(
  currentData.length > 0 ? currentData[0].count : 1
)

// Monochrome zinc theme colors for different wallets
const WALLET_COLORS = [
  "#f4f4f5", // zinc-100 - lightest
  "#e4e4e7", // zinc-300 
  "#d4d4d8", // zinc-300
  "#a1a1aa", // zinc-400
  "#71717a", // zinc-500 - darkest
]

function toggleViewMode() {
  viewMode = viewMode === 'senders' ? 'receivers' : 'senders'
}
</script>

<Card class="h-full">
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <h3 class="text-sm font-medium text-zinc-300">Most Active</h3>
        <span class="text-xs text-zinc-500">Top {currentData.length}</span>
      </div>
      
      <div class="flex items-center gap-3">
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
        
        <!-- View mode toggle -->
        <div class="flex items-center text-xs">
          <button
            onclick={toggleViewMode}
            class="transition-colors {
              viewMode === 'senders' 
                ? 'text-zinc-100 font-medium' 
                : 'text-zinc-500 hover:text-zinc-300'
            }"
          >
            Senders
          </button>
          <span class="mx-1 text-zinc-600">·</span>
          <button
            onclick={toggleViewMode}
            class="transition-colors {
              viewMode === 'receivers' 
                ? 'text-zinc-100 font-medium' 
                : 'text-zinc-500 hover:text-zinc-300'
            }"
          >
            Receivers
          </button>
        </div>
      </div>
    </div>
    
    <div class="flex-1 space-y-2 min-h-0 overflow-y-auto">
      {#each currentData as wallet, index}
        <div class="group">
          <!-- Wallet info -->
          <div class="flex items-center justify-between mb-1">
            <div class="flex items-center gap-2 text-xs">
              <div 
                class="w-2 h-2 rounded-full flex-shrink-0"
                style="background-color: {WALLET_COLORS[index % WALLET_COLORS.length]}"
              ></div>
              <span 
                class="text-zinc-300 font-mono text-xs truncate"
                title={wallet.address}
              >
                {wallet.displayAddress}
              </span>
            </div>
            <span class="text-zinc-400 text-xs flex-shrink-0 ml-2">
              {wallet.count}
            </span>
          </div>
          
          <!-- Progress bar -->
          <div class="w-full bg-zinc-800 rounded-full h-1.5 overflow-hidden">
            <div 
              class="h-full rounded-full transition-all duration-500 ease-out"
              style="
                background-color: {WALLET_COLORS[index % WALLET_COLORS.length]};
                width: {(wallet.count / maxCount) * 100}%;
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
            <!-- Wallet info skeleton -->
            <div class="flex items-center justify-between mb-1">
              <div class="flex items-center gap-2 text-xs">
                <Skeleton class="w-2 h-2 rounded-full" />
                <Skeleton class="h-3 w-24 font-mono" />
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