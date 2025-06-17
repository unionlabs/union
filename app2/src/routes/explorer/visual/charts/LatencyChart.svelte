<script lang="ts">
// Simple latency chart component for displaying chain-to-chain latency data

interface LatencyMetrics {
  p5?: number
  median?: number
  p95?: number
}

interface LatencyDataItem {
  sourceName: string
  destinationName: string
  packetRecv?: LatencyMetrics
  writeAck?: LatencyMetrics
  packetAck?: LatencyMetrics
  [key: string]: any // Allow string indexing for metric access
}

export let latencyData: LatencyDataItem[] = []

// Local configuration
const metricTypes = [
  { key: "packetRecv", label: "Packet Recv", description: "Time until packet received" },
  { key: "writeAck", label: "Write Ack", description: "Time until write acknowledgment" },
  { key: "packetAck", label: "Packet Ack", description: "Time until packet acknowledgment" },
]

const percentileTypes = [
  { key: "median", label: "Median", description: "50th percentile" },
  { key: "p95", label: "P95", description: "95th percentile" },
  { key: "p5", label: "P5", description: "5th percentile" },
]

// State management
let selectedMetric = "packetRecv"
let selectedPercentile = "median"

// Computed values
$: currentData = latencyData && latencyData.length > 0
  ? latencyData
    .map(item => ({
      ...item,
      latencyValue: item[selectedMetric] && item[selectedMetric][selectedPercentile] || 0,
    }))
    .sort((a, b) => a.latencyValue - b.latencyValue) // Sort by latency (lowest first)
    .slice(0, 10) // Show top 10
  : []

$: hasData = currentData.length > 0
$: isLoading = !hasData && (!latencyData || latencyData.length === 0)
$: maxLatency = currentData.length > 0 ? Math.max(...currentData.map(item => item.latencyValue)) : 1

// Utility functions
function formatLatency(seconds: number): string {
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`
  }
  if (seconds < 3600) {
    return `${(seconds / 60).toFixed(1)}m`
  }
  return `${(seconds / 3600).toFixed(1)}h`
}

function formatChainName(name: string): string {
  return name ? name.toLowerCase().replace(/\s+/g, "_") : "unknown"
}

function getLatencyColor(latency: number): string {
  if (latency < 30) {
    return "text-green-400"
  }
  if (latency < 120) {
    return "text-yellow-400"
  }
  return "text-red-400"
}

function getProgressColor(latency: number): string {
  if (latency < 30) {
    return "bg-green-500"
  }
  if (latency < 120) {
    return "bg-yellow-500"
  }
  return "bg-red-500"
}
</script>

<div class="h-full p-0 bg-zinc-950 border border-zinc-800 rounded">
  <div class="flex flex-col h-full font-mono">
    <!-- Terminal Header -->
    <header class="flex items-center justify-between p-2 border-b border-zinc-800">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500 text-xs">$</span>
        <h3 class="text-xs text-zinc-300 font-semibold">latency-stats</h3>
        <span class="text-zinc-600 text-xs">--metric={selectedMetric}</span>
        <span class="text-zinc-600 text-xs">--percentile={selectedPercentile}</span>
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
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-1 mb-1">
        <!-- Metric Selector -->
        <div class="flex flex-wrap gap-0.5">
          <span class="text-zinc-600 text-xs font-mono mr-1">metric:</span>
          {#each metricTypes as metric}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedMetric === metric.key
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              on:click={() => selectedMetric = metric.key}
              title={metric.description}
            >
              {metric.label}
            </button>
          {/each}
        </div>

        <!-- Percentile Selector -->
        <div class="flex items-center gap-0.5">
          <span class="text-zinc-600 text-xs font-mono">percentile:</span>
          {#each percentileTypes as percentile}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedPercentile === percentile.key
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              on:click={() => selectedPercentile = percentile.key}
              title={percentile.description}
            >
              {percentile.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Latency List -->
    <main class="flex-1 flex flex-col p-2">
      <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
        chain_latencies:
      </div>

      <div class="flex-1 flex flex-col">
        {#if isLoading}
          <!-- Loading State -->
          <div class="space-y-0.5 flex-1">
            {#each Array(10) as _, index}
              <div class="p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1 text-xs">
                    <div class="w-2 h-2 bg-zinc-700 rounded animate-pulse"></div>
                    <div class="w-12 h-2 bg-zinc-700 rounded animate-pulse"></div>
                    <span class="text-zinc-600 text-xs">→</span>
                    <div class="w-12 h-2 bg-zinc-700 rounded animate-pulse"></div>
                  </div>
                  <div class="flex items-center space-x-1">
                    <div class="w-8 h-2 bg-zinc-700 rounded animate-pulse"></div>
                  </div>
                </div>
                <div class="flex items-center space-x-2">
                  <div class="flex-1 flex min-w-0">
                    <div class="w-full h-1 bg-zinc-700 rounded animate-pulse"></div>
                  </div>
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
          <!-- Latency Data -->
          <div class="space-y-1 flex-1 overflow-y-auto">
            {#each currentData as item, index}
              <article class="p-2 sm:p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                <!-- Route Header -->
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1 text-xs">
                    <span class="text-zinc-500">#{index + 1}</span>
                    <span class="text-zinc-300 font-medium">
                      {formatChainName(item.sourceName)}
                    </span>
                    <span class="text-zinc-600">→</span>
                    <span class="text-zinc-300 font-medium">
                      {formatChainName(item.destinationName)}
                    </span>
                  </div>
                  <div class="flex items-center space-x-1">
                    <span
                      class="text-xs tabular-nums font-medium {getLatencyColor(item.latencyValue)}"
                    >
                      {formatLatency(item.latencyValue)}
                    </span>
                  </div>
                </div>

                <!-- Progress Bar -->
                <div class="flex items-center space-x-2">
                  <div class="flex-1 flex min-w-0">
                    <!-- Desktop: Thinner bar -->
                    <div class="hidden sm:flex w-full h-1">
                      <div
                        class="h-full transition-all duration-300 {getProgressColor(item.latencyValue)}"
                        style="width: {(item.latencyValue / maxLatency) * 100}%"
                        title="Latency: {formatLatency(item.latencyValue)}"
                      >
                      </div>
                      <div
                        class="bg-zinc-800 h-full transition-all duration-300"
                        style="width: {100 - (item.latencyValue / maxLatency) * 100}%"
                      >
                      </div>
                    </div>
                    <!-- Mobile: Thicker bar for better visibility -->
                    <div class="flex sm:hidden w-full h-1.5">
                      <div
                        class="h-full transition-all duration-300 {getProgressColor(item.latencyValue)}"
                        style="width: {(item.latencyValue / maxLatency) * 100}%"
                        title="Latency: {formatLatency(item.latencyValue)}"
                      >
                      </div>
                      <div
                        class="bg-zinc-800 h-full transition-all duration-300"
                        style="width: {100 - (item.latencyValue / maxLatency) * 100}%"
                      >
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Additional Metrics -->
                {#if item.packetRecv}
                  <div class="mt-1 text-xs text-zinc-500">
                    <span class="mr-2">P5: {formatLatency(item.packetRecv.p5 || 0)}</span>
                    <span class="mr-2">Median: {formatLatency(item.packetRecv.median || 0)}</span>
                    <span>P95: {formatLatency(item.packetRecv.p95 || 0)}</span>
                  </div>
                {/if}
              </article>
            {/each}
          </div>
        {/if}
      </div>
    </main>
  </div>
</div>

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
