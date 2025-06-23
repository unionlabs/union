<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

interface NodeData {
  chainId: string
  chainName: string
  rpcUrl: string
  rpcType: string
  status: string // "healthy", "degraded", "unhealthy"
  responseTimeMs: number
  lastCheckTime: number
  latestBlockHeight?: number
  errorMessage?: string
  uptime: number // Percentage uptime over last 24h
}

interface ChainHealthStat {
  chainName: string
  healthyNodes: number
  totalNodes: number
  avgResponseTime: number
  uptime: number
}

interface NodeHealthSummary {
  totalNodes: number
  healthyNodes: number
  degradedNodes: number
  unhealthyNodes: number
  avgResponseTime: number
  nodesWithRpcs: NodeData[]
  chainHealthStats: Record<string, ChainHealthStat>
  dataAvailability: {
    hasMinute: boolean
    hasHour: boolean
    hasDay: boolean
    has7Days: boolean
    has14Days: boolean
    has30Days: boolean
  }
}

interface Props {
  nodeHealthData?: NodeHealthSummary
}

const DEFAULT_NODE_HEALTH: NodeHealthSummary = {
  totalNodes: 0,
  healthyNodes: 0,
  degradedNodes: 0,
  unhealthyNodes: 0,
  avgResponseTime: 0,
  nodesWithRpcs: [],
  chainHealthStats: {},
  dataAvailability: {
    hasMinute: false,
    hasHour: false,
    hasDay: false,
    has7Days: false,
    has14Days: false,
    has30Days: false,
  }
}

let {
  nodeHealthData = DEFAULT_NODE_HEALTH,
}: Props = $props()

// Local display configuration
const displayOptions = [
  { value: "all", label: "All" },
  { value: "healthy", label: "Healthy" },
  { value: "degraded", label: "Degraded" },
  { value: "unhealthy", label: "Unhealthy" },
]

const sortOptions = [
  { value: "chain", label: "Chain" },
  { value: "status", label: "Status" },
  { value: "response", label: "Response" },
  { value: "uptime", label: "Uptime" },
]

// State management
let selectedFilter = $state("all")
let selectedSort = $state("chain")

// Derived state
const currentData = $derived.by(() => {
  let data = [...(nodeHealthData.nodesWithRpcs || [])] // Create a copy to avoid mutation
  
  // Filter by status
  if (selectedFilter !== "all") {
    data = data.filter(node => node.status === selectedFilter)
  }
  
  // Sort data
  switch (selectedSort) {
    case "status":
      data = data.sort((a, b) => {
        const statusOrder = { healthy: 0, degraded: 1, unhealthy: 2 }
        return statusOrder[a.status] - statusOrder[b.status]
      })
      break
    case "response":
      data = data.sort((a, b) => (a.responseTimeMs || Infinity) - (b.responseTimeMs || Infinity))
      break
    case "uptime":
      data = data.sort((a, b) => b.uptime - a.uptime)
      break
    case "chain":
    default:
      data = data.sort((a, b) => a.chainName.localeCompare(b.chainName))
      break
  }
  
  return data
})

const hasData = $derived(currentData.length > 0)
const isLoading = $derived(!hasData && nodeHealthData.totalNodes === 0)

// Utility functions
function formatResponseTime(ms: number): string {
  if (ms === 0) return "N/A"
  if (ms >= 1000) return `${(ms / 1000).toFixed(1)}s`
  return `${ms}ms`
}

function formatUptime(uptime: number): string {
  return `${uptime.toFixed(1)}%`
}

function formatBlockHeight(height?: number): string {
  if (!height) return "N/A"
  // Show full block height - no abbreviation since it's critical for debugging
  return height.toLocaleString() // Adds commas for readability (e.g., 1,234,567)
}

function getStatusColor(status: string): string {
  switch (status) {
    case "healthy":
      return "text-emerald-400"
    case "degraded":
      return "text-orange-400"
    case "unhealthy":
      return "text-red-400"
    default:
      return "text-zinc-500"
  }
}

function getStatusBgColor(status: string): string {
  switch (status) {
    case "healthy":
      return "bg-emerald-500/20"
    case "degraded":
      return "bg-orange-500/20"
    case "unhealthy":
      return "bg-red-500/20"
    default:
      return "bg-zinc-500/20"
  }
}

function shortenRpcUrl(url: string): string {
  try {
    const urlObj = new URL(url)
    return urlObj.hostname
  } catch {
    return url.length > 30 ? url.substring(0, 30) + "..." : url
  }
}

function formatLastCheckTime(timestamp: number): string {
  const now = Date.now() / 1000
  const diff = now - timestamp
  
  if (diff < 60) return "just now"
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  return `${Math.floor(diff / 86400)}d ago`
}

// Health percentage calculation
const healthPercentage = $derived(() => {
  if (nodeHealthData.totalNodes === 0) return 0
  return Math.round((nodeHealthData.healthyNodes / nodeHealthData.totalNodes) * 100)
})

// Debug logging in development
$effect(() => {
  if (import.meta.env.DEV) {
    console.log("NodeHealthChart data:", {
      hasData,
      isLoading,
      totalNodes: nodeHealthData.totalNodes,
      healthyNodes: nodeHealthData.healthyNodes,
      currentDataLength: currentData.length,
      selectedFilter,
      selectedSort,
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
        <h3 class="text-xs text-zinc-300 font-semibold">node-health</h3>
        <span class="text-zinc-600 text-xs">--monitor=rpc</span>
      </div>
      <div class="text-xs text-zinc-500">
        {#if isLoading}
          loading...
        {:else if !hasData}
          no nodes found
        {:else}
          {healthPercentage}% healthy
        {/if}
      </div>
    </header>

    <!-- Status Summary -->
    {#if hasData}
      <div class="px-2 py-1 border-b border-zinc-800">
        <div class="flex flex-wrap gap-2 text-xs">
          <span class="text-emerald-400">
            ●{nodeHealthData.healthyNodes} healthy
          </span>
          <span class="text-orange-400">
            ●{nodeHealthData.degradedNodes} degraded
          </span>
          <span class="text-red-400">
            ●{nodeHealthData.unhealthyNodes} unhealthy
          </span>
          <span class="text-zinc-500">
            avg: {formatResponseTime(nodeHealthData.avgResponseTime)}
          </span>
        </div>
      </div>
    {/if}

    <!-- Controls -->
    <div class="pt-2 px-2">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-1 mb-1">
        <!-- Filter Selector -->
        <div class="flex items-center gap-0.5">
          <span class="text-zinc-600 text-xs font-mono">filter:</span>
          {#each displayOptions as option}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedFilter === option.value
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              onclick={() => selectedFilter = option.value}
            >
              {option.label}
            </button>
          {/each}
        </div>

        <!-- Sort Selector -->
        <div class="flex items-center gap-0.5">
          <span class="text-zinc-600 text-xs font-mono">sort:</span>
          {#each sortOptions as option}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedSort === option.value
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              onclick={() => selectedSort = option.value}
            >
              {option.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Nodes List -->
    <main class="flex-1 flex flex-col p-2">
      <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
        rpc_nodes:
      </div>

      <div class="flex-1 flex flex-col">
        {#if isLoading}
          <!-- Loading State -->
          <div class="space-y-0.5 flex-1">
            {#each Array(5) as _, index}
              <div class="p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1 text-xs">
                    <Skeleton class="w-2 h-2" />
                    <Skeleton class="w-16 h-2" />
                    <span class="text-zinc-600 text-xs">•</span>
                    <Skeleton class="w-20 h-2" />
                  </div>
                  <div class="flex items-center space-x-1">
                    <Skeleton class="w-8 h-2" />
                    <Skeleton class="w-10 h-2" />
                  </div>
                </div>
                <div class="flex items-center space-x-2">
                  <Skeleton class="w-full h-1" />
                  <Skeleton class="w-8 h-2" />
                </div>
              </div>
            {/each}
          </div>
        {:else if !hasData}
          <!-- No Data State -->
          <div class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <div class="text-zinc-600 font-mono">no_nodes_available</div>
                             <div class="text-zinc-700 text-xs mt-1">
                 {selectedFilter !== "all" ? `no ${selectedFilter} nodes found` : "no nodes available"}
               </div>
            </div>
          </div>
        {:else}
          <!-- Nodes Data -->
          <div class="space-y-1 flex-1 overflow-y-auto">
            {#each currentData as node, index}
              <article class="p-2 sm:p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                <!-- Node Header -->
                <div class="flex items-center justify-between mb-0.5">
                  <div class="flex items-center space-x-1 text-xs">
                    <span class="text-zinc-500">#{index + 1}</span>
                    <span class="text-zinc-300 font-medium">
                      {node.chainName}
                    </span>
                    <span class="text-zinc-600">•</span>
                    <span class="text-zinc-400 text-xs">
                      {node.rpcType}
                    </span>
                  </div>
                  <div class="flex items-center space-x-1">
                    <span class="text-xs text-zinc-500 tabular-nums">
                      {formatLastCheckTime(node.lastCheckTime)}
                    </span>
                  </div>
                </div>

                <!-- RPC URL -->
                <div class="mb-0.5">
                  <a 
                    href={node.rpcUrl} 
                    target="_blank" 
                    rel="noopener noreferrer"
                    class="text-xs text-zinc-400 hover:text-zinc-300 transition-colors font-mono break-all"
                    title={node.rpcUrl}
                  >
                    {shortenRpcUrl(node.rpcUrl)}
                  </a>
                </div>

                <!-- Status and Metrics -->
                <div class="flex items-center justify-between">
                  <div class="flex items-center space-x-2">
                    <!-- Status -->
                    <span 
                      class="px-1.5 py-0.5 text-xs font-medium rounded-sm {getStatusBgColor(node.status)} {getStatusColor(node.status)}"
                      title={node.errorMessage || node.status}
                    >
                      {node.status}
                    </span>

                    <!-- Response Time -->
                    {#if node.responseTimeMs > 0}
                      <span class="text-zinc-500 text-xs tabular-nums">
                        {formatResponseTime(node.responseTimeMs)}
                      </span>
                    {/if}

                    <!-- Block Height -->
                    {#if node.latestBlockHeight}
                      <span class="text-zinc-500 text-xs tabular-nums">
                        #{formatBlockHeight(node.latestBlockHeight)}
                      </span>
                    {/if}
                  </div>

                  <!-- Uptime -->
                  <div class="flex items-center space-x-1">
                    <span class="text-zinc-500 text-xs tabular-nums">
                      {formatUptime(node.uptime)}
                    </span>
                    <div class="w-8 h-1 bg-zinc-800 rounded-full overflow-hidden">
                      <div
                        class="h-full transition-all duration-300 {
                          node.uptime >= 95 ? 'bg-emerald-400' : 
                          node.uptime >= 80 ? 'bg-amber-400' : 'bg-red-400'
                        }"
                        style="width: {Math.min(node.uptime, 100)}%"
                      ></div>
                    </div>
                  </div>
                </div>

                <!-- Error Message -->
                {#if node.errorMessage && node.status === "unhealthy"}
                  <div class="mt-1 text-xs text-red-400 font-mono break-all">
                    error: {node.errorMessage}
                  </div>
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