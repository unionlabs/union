<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { Option, pipe } from "effect"
import type { ChainAsset, ChainData } from "../types"
interface ChainFlowData {
  chains: ChainData[]
  chainFlowTimeScale: Record<string, ChainData[]>
  totalOutgoing: number
  totalIncoming: number
  serverUptimeSeconds: number
}
interface Props {
  chainFlowData: Option.Option<ChainFlowData>
}

let {
  chainFlowData,
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
let displayMode = $state<"total" | "outgoing" | "incoming" | "netflow">("total")
let expandedChain = $state<string | null>(null)
let hoveredChain = $state<string | null>(null)
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

// Display mode configuration
const displayModes = [
  { key: "total", label: "total" },
  { key: "outgoing", label: "out" },
  { key: "incoming", label: "in" },
  { key: "netflow", label: "net" },
] as const

// Get current data based on selected time scale using Effect Option patterns
const currentData = $derived.by(() => {
  return pipe(
    chainFlowData,
    Option.map((data) => {
      let chainData = []
      if (
        data.chainFlowTimeScale
        && data.chainFlowTimeScale[selectedTimeScale]
        && data.chainFlowTimeScale[selectedTimeScale].length > 0
      ) {
        chainData = data.chainFlowTimeScale[selectedTimeScale]
      } else {
        chainData = data.chains
      }

      // Sort based on display mode
      const sortedData = [...(chainData || [])].sort((a, b) => {
        switch (displayMode) {
          case "outgoing":
            return b.outgoingCount - a.outgoingCount
          case "incoming":
            return b.incomingCount - a.incomingCount
          case "netflow":
            return calculateNetFlow(b) - calculateNetFlow(a)
          case "total":
          default:
            return (b.outgoingCount + b.incomingCount) - (a.outgoingCount + a.incomingCount)
        }
      })

      // Limit to selected number for display
      return sortedData?.slice(0, selectedItemCount) || []
    }),
    Option.getOrElse(() => []),
  )
})

// Derived state
const hasData = $derived(currentData.length > 0)
const isLoading = $derived(
  !hasData && Option.isNone(chainFlowData),
)

// Utility functions
function formatCount(count: number): string {
  if (count >= 1000000) {
    return `${(count / 1000000).toFixed(1)}M`
  }
  if (count >= 1000) {
    return `${(count / 1000).toFixed(1)}K`
  }
  return count.toString().padStart(3, " ")
}

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

function formatTime(dateString: string): string {
  try {
    return new Date(dateString).toLocaleTimeString()
  } catch {
    return "N/A"
  }
}

// Get display value based on mode
function getDisplayValue(chain: ChainData): number {
  switch (displayMode) {
    case "outgoing":
      return chain.outgoingCount
    case "incoming":
      return chain.incomingCount
    case "netflow":
      return calculateNetFlow(chain)
    case "total":
    default:
      return chain.outgoingCount + chain.incomingCount
  }
}

// Get change value based on mode
function getChangeValue(chain: ChainData): number | undefined {
  switch (displayMode) {
    case "outgoing":
      return chain.outgoingChange
    case "incoming":
      return chain.incomingChange
    case "netflow":
      return chain.netFlowChange
    case "total":
    default:
      // For total, we could average the changes or use a different logic
      return chain.outgoingChange !== undefined && chain.incomingChange !== undefined
        ? (chain.outgoingChange + chain.incomingChange) / 2
        : undefined
  }
}

// Toggle chain expansion
function toggleChainExpansion(chainId: string): void {
  expandedChain = expandedChain === chainId ? null : chainId
}

// Check if chain should show colored bars (expanded, hovered, or parent of expanded)
function shouldShowColoredBars(chainId: string): boolean {
  return expandedChain === chainId || hoveredChain === chainId
}

// Get bar colors based on state
function getBarColors(chainId: string): { incoming: string; outgoing: string } {
  if (shouldShowColoredBars(chainId)) {
    return {
      incoming: "bg-green-400",
      outgoing: "bg-red-400",
    }
  }
  return {
    incoming: "bg-zinc-400",
    outgoing: "bg-zinc-500",
  }
}

// Calculate incoming/outgoing widths for assets
function getAssetIncomingOutgoingWidths(
  asset: ChainAsset,
  totalChars: number = 50,
): { incomingWidth: number; outgoingWidth: number } {
  const totalFlow = asset.incomingCount + asset.outgoingCount
  if (totalFlow === 0) {
    return { incomingWidth: 0, outgoingWidth: totalChars }
  }

  // Calculate proportional widths based on actual flow ratios
  const incomingRatio = asset.incomingCount / totalFlow

  const incomingWidth = Math.round(totalChars * incomingRatio)
  const outgoingWidth = totalChars - incomingWidth // Ensure they add up to exactly totalChars

  return {
    incomingWidth: Math.max(0, incomingWidth),
    outgoingWidth: Math.max(0, outgoingWidth),
  }
}

// Progress bar calculations for incoming/outgoing split with dynamic center
function getIncomingOutgoingWidths(
  chain: ChainData,
  totalChars: number = 50,
): { incomingWidth: number; outgoingWidth: number } {
  if (currentData.length === 0) {
    return { incomingWidth: 0, outgoingWidth: totalChars }
  }

  const totalFlow = chain.incomingCount + chain.outgoingCount
  if (totalFlow === 0) {
    return { incomingWidth: 0, outgoingWidth: totalChars }
  }

  // Calculate proportional widths based on actual flow ratios - always use full width
  const incomingRatio = chain.incomingCount / totalFlow

  const incomingWidth = Math.round(totalChars * incomingRatio)
  const outgoingWidth = totalChars - incomingWidth // Ensure they add up to exactly totalChars

  return {
    incomingWidth: Math.max(0, incomingWidth),
    outgoingWidth: Math.max(0, outgoingWidth),
  }
}

function getPercentageOfTotal(chain: ChainData): number {
  if (currentData.length === 0) {
    return 0
  }

  const displayValue = Math.abs(getDisplayValue(chain))
  const total = currentData.reduce((sum, c) => sum + Math.abs(getDisplayValue(c)), 0)

  if (total === 0) {
    return 0
  }

  return Math.round((displayValue / total) * 100)
}

// Calculate correct net flow: incoming - outgoing (proper accounting)
function calculateNetFlow(chain: ChainData): number {
  return chain.incomingCount - chain.outgoingCount
}

// Calculate correct net flow for assets: incoming - outgoing (proper accounting)
function calculateAssetNetFlow(asset: ChainAsset): number {
  return asset.incomingCount - asset.outgoingCount
}

// Debug logging in development
$effect(() => {
  if (typeof window !== "undefined" && window.location.hostname === "localhost") {
    console.log("ChainFlowChart data:", {
      hasChainFlowData: Option.isSome(chainFlowData),
      hasData,
      isLoading,
      currentDataLength: currentData.length,
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
        <h3 class="text-xs text-zinc-300 font-semibold">chain-flow</h3>
        <span class="text-zinc-600 text-xs">--tf={selectedTimeScale}</span>
        <span class="text-zinc-600 text-xs">--mode={displayMode}</span>
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
      <!-- Mobile: Stack controls, Tablet+: Multi-row layout -->
      <div class="flex flex-col gap-2 mb-1">
        <!-- Top Row: Time Frame and Item Count -->
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-1">
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

        <!-- Bottom Row: Display Mode Selector -->
        <div class="flex items-center gap-0.5">
          <span class="text-zinc-600 text-xs font-mono">view:</span>
          {#each displayModes as mode}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                displayMode === mode.key
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              onclick={() => displayMode = mode.key}
            >
              {mode.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Chain Flow List -->
    <main class="flex-1 flex flex-col p-2">
      <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
        chain_flows:
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
                    <Skeleton class="w-16 h-2" />
                  </div>
                  <Skeleton class="w-10 h-2" />
                </div>
                <div class="flex items-center space-x-1.5 text-xs mb-0.5">
                  <span class="text-zinc-600">in:</span>
                  <Skeleton class="w-6 h-2" />
                  <span class="text-zinc-600">out:</span>
                  <Skeleton class="w-6 h-2" />
                  <span class="text-zinc-600">net:</span>
                  <Skeleton class="w-6 h-2" />
                  <Skeleton class="w-6 h-2" />
                  <span class="text-zinc-600">assets</span>
                </div>
                <div class="flex items-center">
                  <div class="flex-1 flex min-w-0">
                    <div class="hidden md:flex w-full h-1">
                      <Skeleton class="w-full h-1" />
                    </div>
                    <div class="flex md:hidden w-full h-1">
                      <Skeleton class="w-full h-1" />
                    </div>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else if !hasData}
          <!-- No Data State -->
          <div class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <div class="text-zinc-600 font-mono">
                {#if Option.isSome(chainFlowData)}
                  {
                    pipe(
                      chainFlowData,
                      Option.map(data =>
                        data.chains?.length > 0
                          ? `no_data_yet • ${data.chains.length} chains available`
                          : "no_chain_data_available"
                      ),
                      Option.getOrElse(() => "waiting_for_server..."),
                    )
                  }
                {:else}
                  waiting_for_server...
                {/if}
              </div>
            </div>
          </div>
        {:else}
          <!-- Chain Flow Data -->
          <div class="space-y-1 flex-1 overflow-y-auto">
            {#each currentData as chain, index}
              <article class="bg-zinc-900 border border-zinc-800 rounded transition-colors">
                <!-- Main Chain Row -->
                <button
                  class="w-full p-2 sm:p-1.5 text-left hover:bg-zinc-800 transition-colors"
                  onclick={() => toggleChainExpansion(chain.universal_chain_id)}
                  onmouseenter={() => hoveredChain = chain.universal_chain_id}
                  onmouseleave={() => hoveredChain = null}
                  aria-expanded={expandedChain === chain.universal_chain_id}
                  aria-controls="assets-{chain.universal_chain_id}"
                >
                  <!-- Chain Header -->
                  <div class="flex items-center justify-between mb-0.5">
                    <div class="flex items-center space-x-1 text-xs">
                      <span class="text-zinc-500">#{index + 1}</span>
                      <span class="text-zinc-200 font-bold">
                        {formatChainName(chain.chainName)}
                      </span>
                      <span class="text-zinc-600 text-[10px]">({chain.universal_chain_id})</span>
                      {#if chain.topAssets && chain.topAssets.length > 0}
                        <span class="text-zinc-600 text-xs">
                          {expandedChain === chain.universal_chain_id ? "▼" : "▶"}
                          {chain.topAssets.length}
                        </span>
                      {/if}
                    </div>
                    <div class="text-zinc-300 font-mono font-medium text-xs">
                      {#if getChangeValue(chain) !== undefined}
                        <span
                          class="text-xs mr-1 hidden md:inline {getChangeValue(chain)! >= 0 ? 'text-green-400' : 'text-red-400'}"
                        >{formatPercentageChange(getChangeValue(chain))}</span>
                      {/if}
                      {formatCount(getDisplayValue(chain))}
                    </div>
                  </div>

                  <!-- Flow Details -->
                  <div class="flex items-center space-x-1.5 text-xs mb-0.5">
                    <span class="text-zinc-600">in:</span>
                    <span class="text-zinc-400 tabular-nums">{
                      formatCount(chain.incomingCount)
                    }</span>
                    <span class="text-zinc-600">out:</span>
                    <span class="text-zinc-400 tabular-nums">{
                      formatCount(chain.outgoingCount)
                    }</span>
                    <span class="text-zinc-600">net:</span>
                    <span
                      class="tabular-nums {calculateNetFlow(chain) >= 0 ? 'text-green-400' : 'text-red-400'}"
                    >
                      {calculateNetFlow(chain) >= 0 ? "+" : ""}{
                        formatCount(calculateNetFlow(chain))
                      }
                    </span>
                    {#if chain.topAssets && chain.topAssets.length > 0}
                      <span class="text-zinc-600">{chain.topAssets.length}</span>
                      <span class="text-zinc-600">assets</span>
                    {/if}
                  </div>

                  <!-- Chain Flow Visualization -->
                  <div class="flex items-center">
                    <div class="flex-1 flex min-w-0">
                      <div class="hidden sm:flex w-full h-1">
                        <!-- Incoming side (left, green) -->
                        {#if getIncomingOutgoingWidths(chain, 50).incomingWidth > 0}
                          <div
                            class="{getBarColors(chain.universal_chain_id).incoming} h-full transition-colors duration-300"
                            style="width: {(getIncomingOutgoingWidths(chain, 50).incomingWidth / 50) * 100}%"
                            title="Incoming: {chain.incomingCount}"
                          >
                          </div>
                        {/if}
                        <!-- Outgoing side (right, red) -->
                        {#if getIncomingOutgoingWidths(chain, 50).outgoingWidth > 0}
                          <div
                            class="{getBarColors(chain.universal_chain_id).outgoing} h-full transition-colors duration-300"
                            style="width: {(getIncomingOutgoingWidths(chain, 50).outgoingWidth / 50) * 100}%"
                            title="Outgoing: {chain.outgoingCount}"
                          >
                          </div>
                        {/if}
                      </div>
                      <div class="flex sm:hidden w-full h-1.5">
                        <!-- Incoming side (left, green) -->
                        {#if getIncomingOutgoingWidths(chain, 25).incomingWidth > 0}
                          <div
                            class="{getBarColors(chain.universal_chain_id).incoming} h-full transition-colors duration-300"
                            style="width: {(getIncomingOutgoingWidths(chain, 25).incomingWidth / 25) * 100}%"
                            title="Incoming: {chain.incomingCount}"
                          >
                          </div>
                        {/if}
                        <!-- Outgoing side (right, red) -->
                        {#if getIncomingOutgoingWidths(chain, 25).outgoingWidth > 0}
                          <div
                            class="{getBarColors(chain.universal_chain_id).outgoing} h-full transition-colors duration-300"
                            style="width: {(getIncomingOutgoingWidths(chain, 25).outgoingWidth / 25) * 100}%"
                            title="Outgoing: {chain.outgoingCount}"
                          >
                          </div>
                        {/if}
                      </div>
                    </div>
                    {#if getChangeValue(chain) !== undefined}
                      <span
                        class="text-[11px] sm:text-[10px] tabular-nums sm:hidden ml-2 {getChangeValue(chain)! >= 0 ? 'text-green-400' : 'text-red-400'}"
                      >{formatPercentageChange(getChangeValue(chain))}</span>
                    {/if}
                  </div>
                </button>

                <!-- Expanded Assets Section -->
                {#if expandedChain === chain.universal_chain_id && chain.topAssets
                && chain.topAssets.length > 0}
                  <section
                    id="assets-{chain.universal_chain_id}"
                    class="border-t border-zinc-800 bg-zinc-950 p-1.5"
                  >
                    <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
                      assets for {formatChainName(chain.chainName)}:
                    </div>
                    <div class="space-y-0.5">
                      {#each chain.topAssets as asset, assetIndex}
                        <div class="bg-zinc-900 p-1 rounded border border-zinc-800">
                          <div class="flex items-center justify-between text-xs mb-0.5">
                            <div class="flex items-center space-x-1">
                              <span class="text-zinc-600">#{assetIndex + 1}</span>
                              <span class="text-zinc-200 font-bold">{asset.assetSymbol}</span>
                              {#if asset.assetName
                          && asset.assetName !== asset.assetSymbol}
                                <span class="text-zinc-500">({asset.assetName})</span>
                              {/if}
                            </div>
                            <div class="flex items-center space-x-1">
                              <span class="text-zinc-600">vol:</span>
                              <span class="text-zinc-400">{formatNumber(asset.totalVolume)}</span>
                              <span class="text-zinc-600">cnt:</span>
                              <span class="text-zinc-400">{
                                asset.outgoingCount + asset.incomingCount
                              }</span>
                              <span class="text-zinc-600">avg:</span>
                              <span class="text-zinc-400">{formatNumber(asset.averageAmount)}</span>
                            </div>
                          </div>

                          <!-- Asset Flow Details -->
                          <div class="flex items-center space-x-1.5 text-xs mb-0.5">
                            <span class="text-zinc-600">in:</span>
                            <span class="text-zinc-400">{formatCount(asset.incomingCount)}</span>
                            <span class="text-zinc-600">out:</span>
                            <span class="text-zinc-400">{formatCount(asset.outgoingCount)}</span>
                            <span class="text-zinc-600">net:</span>
                            <span
                              class="text-zinc-400 {calculateAssetNetFlow(asset) >= 0 ? 'text-green-400' : 'text-red-400'}"
                            >
                              {calculateAssetNetFlow(asset) >= 0 ? "+" : ""}{
                                formatCount(calculateAssetNetFlow(asset))
                              }
                            </span>
                            <span class="text-zinc-600">avg:</span>
                            <span class="text-zinc-400">{formatNumber(asset.averageAmount)}</span>
                          </div>

                          <!-- Asset Flow Bar (Incoming/Outgoing) -->
                          <div class="flex items-center">
                            <div class="flex-1 flex min-w-0">
                              <div class="hidden sm:flex w-full h-1">
                                <!-- Incoming side (left, green) -->
                                {#if getAssetIncomingOutgoingWidths(asset, 50)
                            .incomingWidth > 0}
                                  <div
                                    class="{getBarColors(chain.universal_chain_id).incoming} h-full transition-colors duration-300"
                                    style="width: {(getAssetIncomingOutgoingWidths(asset, 50).incomingWidth / 50) * 100}%"
                                    title="Incoming: {asset.incomingCount}"
                                  >
                                  </div>
                                {/if}
                                <!-- Outgoing side (right, red) -->
                                {#if getAssetIncomingOutgoingWidths(asset, 50)
                            .outgoingWidth > 0}
                                  <div
                                    class="{getBarColors(chain.universal_chain_id).outgoing} h-full transition-colors duration-300"
                                    style="width: {(getAssetIncomingOutgoingWidths(asset, 50).outgoingWidth / 50) * 100}%"
                                    title="Outgoing: {asset.outgoingCount}"
                                  >
                                  </div>
                                {/if}
                              </div>
                              <div class="flex sm:hidden w-full h-1.5">
                                <!-- Incoming side (left, green) -->
                                {#if getAssetIncomingOutgoingWidths(asset, 25)
                            .incomingWidth > 0}
                                  <div
                                    class="{getBarColors(chain.universal_chain_id).incoming} h-full transition-colors duration-300"
                                    style="width: {(getAssetIncomingOutgoingWidths(asset, 25).incomingWidth / 25) * 100}%"
                                    title="Incoming: {asset.incomingCount}"
                                  >
                                  </div>
                                {/if}
                                <!-- Outgoing side (right, red) -->
                                {#if getAssetIncomingOutgoingWidths(asset, 25)
                            .outgoingWidth > 0}
                                  <div
                                    class="{getBarColors(chain.universal_chain_id).outgoing} h-full transition-colors duration-300"
                                    style="width: {(getAssetIncomingOutgoingWidths(asset, 25).outgoingWidth / 25) * 100}%"
                                    title="Outgoing: {asset.outgoingCount}"
                                  >
                                  </div>
                                {/if}
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
