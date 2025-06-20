<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

interface AssetRoute {
  fromChain: string
  toChain: string
  fromName: string
  toName: string
  route: string
  count: number
  volume: number
  percentage: number
  lastActivity: string
}

interface Asset {
  assetSymbol: string
  assetName: string
  transferCount: number
  totalVolume: number
  averageAmount: number
  largestTransfer: number
  volumeChange?: number
  countChange?: number
  lastActivity: string
  topRoutes: AssetRoute[]
}

interface AssetVolumeData {
  assets: Asset[]
  assetVolumeTimeScale: Record<string, Asset[]>
  totalAssets: number
  totalVolume: number
  totalTransfers: number
  serverUptimeSeconds: number
}

interface DataAvailability {
  hasMinute: boolean
  hasHour: boolean
  hasDay: boolean
  has7Days: boolean
  has14Days: boolean
  has30Days: boolean
}

interface Props {
  assetVolumeData?: AssetVolumeData
  dataAvailability?: DataAvailability
}

const DEFAULT_ASSET_DATA: AssetVolumeData = {
  assets: [],
  assetVolumeTimeScale: {},
  totalAssets: 0,
  totalVolume: 0,
  totalTransfers: 0,
  serverUptimeSeconds: 0,
}

const DEFAULT_DATA_AVAILABILITY: DataAvailability = {
  hasMinute: false,
  hasHour: false,
  hasDay: false,
  has7Days: false,
  has14Days: false,
  has30Days: false,
}

let {
  assetVolumeData = DEFAULT_ASSET_DATA,
  dataAvailability = DEFAULT_DATA_AVAILABILITY,
}: Props = $props()

// Local item count configuration
const itemCounts = [
  { value: 3, label: "3" },
  { value: 5, label: "5" },
  { value: 7, label: "7" },
  { value: 10, label: "10" },
]

// State management
let selectedTimeScale = $state("1m")
let expandedAsset = $state<string | null>(null)
let hoveredAsset = $state<string | null>(null)
let selectedItemCount = $state(5) // Default to 5 items

// Time scale configuration
const timeScales = [
  { key: "1m", label: "1m" },
  { key: "1h", label: "1h" },
  { key: "1d", label: "1d" },
  { key: "7d", label: "7d" },
  { key: "14d", label: "14d" },
  { key: "30d", label: "30d" },
] as const

// Route visualization colors
const routeColors = [
  "bg-blue-500",
  "bg-green-500",
  "bg-yellow-500",
  "bg-purple-500",
  "bg-red-500",
] as const

const routeColorsBright = [
  "bg-blue-400",
  "bg-green-400",
  "bg-yellow-400",
  "bg-purple-400",
  "bg-red-400",
] as const

// Derived state
const currentData = $derived.by(() => {
  const data = assetVolumeData?.assets || []
  const sortedData = [...data].sort((a, b) => b.transferCount - a.transferCount)
  return sortedData.slice(0, selectedItemCount)
})

const hasData = $derived(currentData.length > 0)
const isLoading = $derived(!assetVolumeData || assetVolumeData.assets === undefined)

// Utility functions
function formatNumber(num: number): string {
  if (num === 0) {
    return "0"
  }
  if (num < 1) {
    return num.toFixed(6)
  }
  if (num < 1000) {
    return num.toFixed(2)
  }
  if (num < 1000000) {
    return (num / 1000).toFixed(1) + "K"
  }
  if (num < 1000000000) {
    return (num / 1000000).toFixed(1) + "M"
  }
  return (num / 1000000000).toFixed(1) + "B"
}

function formatPercentage(num: number): string {
  if (num === 0) {
    return "0.00%"
  }
  return (num > 0 ? "+" : "") + num.toFixed(2) + "%"
}

function formatTime(dateString: string): string {
  try {
    return new Date(dateString).toLocaleTimeString()
  } catch {
    return "N/A"
  }
}

function getDisplayValue(asset: Asset): string {
  return asset.transferCount.toString()
}

function getChangeValue(asset: Asset): number {
  return asset.countChange || 0
}

function getPercentageOfTotal(asset: Asset): number {
  if (currentData.length === 0) {
    return 0
  }

  const numValue = asset.transferCount
  const total = currentData.reduce((sum, a) => sum + a.transferCount, 0)

  return total === 0 ? 0 : Math.round((numValue / total) * 100)
}

function isTimeFrameAvailable(timeFrameKey: string): boolean {
  // For now, always return true for 1m since that's what we're using
  // TODO: Implement proper time scale data availability based on dataAvailability prop
  return timeFrameKey === "1m"
}

function getFirstAvailableTimeframe(): string {
  for (const timeScale of timeScales) {
    if (isTimeFrameAvailable(timeScale.key)) {
      return timeScale.key
    }
  }
  return "1m"
}

function toggleAssetExpansion(assetSymbol: string): void {
  expandedAsset = expandedAsset === assetSymbol ? null : assetSymbol
}

// Check if asset should show colored bars (expanded or hovered)
function shouldShowColoredBars(assetSymbol: string): boolean {
  return expandedAsset === assetSymbol || hoveredAsset === assetSymbol
}

// Get route colors based on state - grayscale by default, colored when focused
function getRouteColors(assetSymbol: string): string[] {
  if (shouldShowColoredBars(assetSymbol)) {
    return routeColors as unknown as string[]
  }
  return [
    "bg-zinc-400",
    "bg-zinc-500",
    "bg-zinc-600",
    "bg-zinc-500",
    "bg-zinc-400",
  ]
}

function getRouteColorsBright(assetSymbol: string): string[] {
  if (shouldShowColoredBars(assetSymbol)) {
    return routeColorsBright as unknown as string[]
  }
  return [
    "bg-zinc-300",
    "bg-zinc-400",
    "bg-zinc-500",
    "bg-zinc-400",
    "bg-zinc-300",
  ]
}

function calculateRemainingRoutePercentage(routes: AssetRoute[]): number {
  return routes.slice(5).reduce((sum, route) => sum + route.percentage, 0)
}

// Auto-update selected timeframe when data becomes available
$effect(() => {
  const firstAvailable = getFirstAvailableTimeframe()
  if (!isTimeFrameAvailable(selectedTimeScale)) {
    selectedTimeScale = firstAvailable
  }
})

// Debug logging in development
$effect(() => {
  if (import.meta.env.DEV) {
    console.log("AssetVolumeChart data:", {
      hasData,
      isLoading,
      currentDataLength: currentData.length,
      assetsLength: assetVolumeData?.assets?.length || 0,
      totalAssets: assetVolumeData?.totalAssets || 0,
      selectedItemCount: selectedItemCount,
    })
  }
})
</script>

<Card class="h-full p-0">
  <div class="flex flex-col h-full font-mono">
    <!-- Terminal Header -->
    <header class="flex items-center justify-between p-2 border-b border-zinc-800">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500 text-xs">$</span>
        <h3 class="text-xs text-zinc-300 font-semibold">asset-volume</h3>
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
                : isTimeFrameAvailable(timeScale.key)
                ? 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                : 'border-zinc-800 bg-zinc-950 text-zinc-600 cursor-not-allowed'
                }
              "
              disabled={!isTimeFrameAvailable(timeScale.key)}
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

    <!-- Asset Volume List -->
    <main class="flex-1 flex flex-col p-2">
      <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
        asset_flows:
      </div>

      <div class="flex-1 flex flex-col">
        {#if isLoading}
          <!-- Loading State -->
          <div class="space-y-0.5 flex-1">
            {#each Array(selectedItemCount) as _, index}
              <div class="p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1">
                    <Skeleton class="w-2 h-2" />
                    <Skeleton class="w-12 h-2" />
                  </div>
                  <Skeleton class="w-8 h-2" />
                </div>
                <div class="flex items-center space-x-1.5 text-xs mb-0.5">
                  <span class="text-zinc-600">vol:</span>
                  <Skeleton class="w-8 h-2" />
                  <span class="text-zinc-600">cnt:</span>
                  <Skeleton class="w-6 h-2" />
                  <span class="text-zinc-600">avg:</span>
                  <Skeleton class="w-8 h-2" />
                </div>
                <div class="flex items-center space-x-2">
                  <Skeleton class="flex-1 h-1" />
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
          <!-- Asset List -->
          <div class="space-y-1 flex-1 overflow-y-auto">
            {#each currentData as asset, index}
              <article class="bg-zinc-900 border border-zinc-800 rounded transition-colors">
                <!-- Main Asset Row -->
                <button
                  class="w-full p-2 sm:p-1.5 text-left hover:bg-zinc-800 transition-colors"
                  onclick={() => toggleAssetExpansion(asset.assetSymbol)}
                  onmouseenter={() => hoveredAsset = asset.assetSymbol}
                  onmouseleave={() => hoveredAsset = null}
                  aria-expanded={expandedAsset === asset.assetSymbol}
                  aria-controls="routes-{asset.assetSymbol}"
                >
                  <!-- Asset Header -->
                  <div class="flex items-center justify-between mb-0.5">
                    <div class="flex items-center space-x-1">
                      <span class="text-zinc-500 text-xs">#{index + 1}</span>
                      <span class="text-zinc-200 font-bold text-xs">{asset.assetSymbol}</span>
                      {#if asset.assetName && asset.assetName !== asset.assetSymbol}
                        <span class="text-zinc-500 text-xs">({asset.assetName})</span>
                      {/if}
                      {#if asset.topRoutes && asset.topRoutes.length > 0}
                        <span class="text-zinc-600 text-xs">
                          {expandedAsset === asset.assetSymbol ? "▼" : "▶"} {asset.topRoutes.length}
                        </span>
                      {/if}
                    </div>
                    <div class="text-zinc-300 font-mono font-medium text-xs">
                      {getDisplayValue(asset)}
                    </div>
                  </div>

                  <!-- Asset Metrics -->
                  <div class="flex items-center space-x-1.5 text-xs mb-0.5">
                    <span class="text-zinc-600">vol:</span>
                    <span class="text-zinc-400">{formatNumber(asset.totalVolume)}</span>
                    <span class="text-zinc-600">cnt:</span>
                    <span class="text-zinc-400">{asset.transferCount}</span>
                    <span class="text-zinc-600">avg:</span>
                    <span class="text-zinc-400">{formatNumber(asset.averageAmount)}</span>
                  </div>

                  <!-- Route Distribution or Percentage -->
                  {#if asset.topRoutes && asset.topRoutes.length > 0}
                    <div class="flex items-center space-x-2">
                      <div class="flex-1 flex min-w-0 h-1.5 sm:h-1">
                        {#each asset.topRoutes.slice(0, 5) as route, routeIndex}
                          <div
                            class="h-full {getRouteColors(asset.assetSymbol)[routeIndex]} transition-all duration-300"
                            style="width: {route.percentage}%"
                            title="{route.route}: {route.percentage.toFixed(1)}%"
                          >
                          </div>
                        {/each}
                        {#if asset.topRoutes.length > 5}
                          <div
                            class="h-full {shouldShowColoredBars(asset.assetSymbol) ? 'bg-zinc-600' : 'bg-zinc-500'} transition-all duration-300"
                            style="width: {calculateRemainingRoutePercentage(asset.topRoutes)}%"
                            title="Other routes: {calculateRemainingRoutePercentage(asset.topRoutes).toFixed(1)}%"
                          >
                          </div>
                        {/if}
                      </div>
                    </div>
                  {:else}
                    <div class="flex items-center space-x-1">
                      <span class="text-zinc-600 text-xs">share:</span>
                      <span class="text-zinc-500 text-xs tabular-nums font-medium">
                        {getPercentageOfTotal(asset)}%
                      </span>
                      {#if getChangeValue(asset) !== 0}
                        <span
                          class="text-xs tabular-nums ml-1 font-medium {getChangeValue(asset) >= 0 ? 'text-green-400' : 'text-red-400'}"
                        >{formatPercentage(getChangeValue(asset))}</span>
                      {/if}
                    </div>
                  {/if}
                </button>

                <!-- Expanded Routes Section -->
                {#if expandedAsset === asset.assetSymbol && asset.topRoutes
                && asset.topRoutes.length > 0}
                  <section
                    id="routes-{asset.assetSymbol}"
                    class="border-t border-zinc-800 bg-zinc-950 p-1.5"
                  >
                    <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
                      routes for {asset.assetSymbol}:
                    </div>
                    <div class="space-y-0.5">
                      {#each asset.topRoutes as route, routeIndex}
                        <div class="bg-zinc-900 p-1 rounded border border-zinc-800">
                          <div class="flex items-center justify-between text-xs mb-0.5">
                            <div class="flex items-center space-x-1">
                              <span class="text-zinc-600">#{routeIndex + 1}</span>
                              <span class="text-zinc-300 font-medium">{route.route}</span>
                            </div>
                            <div class="flex items-center space-x-1">
                              <span class="text-zinc-600">vol:</span>
                              <span class="text-zinc-400">{formatNumber(route.volume)}</span>
                              <span class="text-zinc-600">cnt:</span>
                              <span class="text-zinc-400">{route.count}</span>
                              <span class="text-zinc-600">%:</span>
                              <span class="text-zinc-400 font-medium">{
                                  route.percentage.toFixed(1)
                                }%</span>
                            </div>
                          </div>
                          <!-- Route Percentage Bar -->
                          <div class="flex items-center space-x-2">
                            <div class="flex-1 flex min-w-0 h-1.5 sm:h-1">
                              <div
                                class="h-full {getRouteColorsBright(asset.assetSymbol)[routeIndex % getRouteColorsBright(asset.assetSymbol).length]} transition-all duration-300"
                                style="width: {route.percentage}%"
                              >
                              </div>
                              <div
                                class="h-full bg-zinc-800 transition-all duration-300"
                                style="width: {100 - route.percentage}%"
                              >
                              </div>
                            </div>
                          </div>
                        </div>
                      {/each}
                    </div>
                  </section>
                {/if}
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
