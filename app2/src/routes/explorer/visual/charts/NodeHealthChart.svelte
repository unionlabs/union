<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { Option, pipe } from "effect"
import type { NodeHealthSummary } from "../types"

interface Props {
  nodeHealthData: Option.Option<NodeHealthSummary>
}

let { nodeHealthData }: Props = $props()

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
]

// State management
let selectedFilter = $state("all")
let selectedSort = $state("chain")

// Derived state
const currentData = $derived.by(() => {
  return pipe(
    nodeHealthData,
    Option.match({
      onNone: () => [],
      onSome: (data) => {
        let nodes = [...data.nodesWithRpcs] // Create a copy to avoid mutation

        // Filter by status
        if (selectedFilter !== "all") {
          nodes = nodes.filter(node => node.status === selectedFilter)
        }

        // Sort data
        switch (selectedSort) {
          case "status":
            nodes = nodes.sort((a, b) => {
              const statusOrder: Record<string, number> = { healthy: 0, degraded: 1, unhealthy: 2 }
              return (statusOrder[a.status] ?? 999) - (statusOrder[b.status] ?? 999)
            })
            break
          case "response":
            nodes = nodes.sort((a, b) =>
              (a.responseTimeMs || Infinity) - (b.responseTimeMs || Infinity)
            )
            break
          case "chain":
          default:
            nodes = nodes.sort((a, b) => a.chainName.localeCompare(b.chainName))
            break
        }

        return nodes
      },
    }),
  )
})

const hasData = $derived(currentData.length > 0)
const isLoading = $derived(
  !hasData && Option.isNone(nodeHealthData),
)

// Utility functions
function formatResponseTime(ms: number): string {
  if (ms === 0) {
    return "N/A"
  }
  if (ms >= 1000) {
    return `${(ms / 1000).toFixed(1)}s`
  }
  return `${ms}ms`
}

function formatBlockHeight(height?: number): string {
  if (!height) {
    return "N/A"
  }
  return height.toLocaleString()
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

  if (diff < 60) {
    return "just now"
  }
  if (diff < 3600) {
    return `${Math.floor(diff / 60)}m ago`
  }
  if (diff < 86400) {
    return `${Math.floor(diff / 3600)}h ago`
  }
  return `${Math.floor(diff / 86400)}d ago`
}

// Health percentage calculation
const healthPercentage = $derived(() => {
  return pipe(
    nodeHealthData,
    Option.match({
      onNone: () => 0,
      onSome: (data) => {
        if (data.totalNodes === 0) {
          return 0
        }
        return Math.round((data.healthyNodes / data.totalNodes) * 100)
      },
    }),
  )
})

// Helper derived values for template
const healthyNodes = $derived(
  pipe(nodeHealthData, Option.getOrElse(() => ({ healthyNodes: 0 })), (data) => data.healthyNodes),
)
const degradedNodes = $derived(
  pipe(
    nodeHealthData,
    Option.getOrElse(() => ({ degradedNodes: 0 })),
    (data) => data.degradedNodes,
  ),
)
const unhealthyNodes = $derived(
  pipe(
    nodeHealthData,
    Option.getOrElse(() => ({ unhealthyNodes: 0 })),
    (data) => data.unhealthyNodes,
  ),
)
const avgResponseTime = $derived(
  pipe(
    nodeHealthData,
    Option.getOrElse(() => ({ avgResponseTime: 0 })),
    (data) => data.avgResponseTime,
  ),
)
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
        {#if !hasData}
          loading...
        {/if}
      </div>
    </header>

    <!-- Status Summary -->
    {#if hasData}
      <div class="px-2 py-1 border-b border-zinc-800">
        <div class="flex flex-wrap gap-2 text-xs">
          <span class="text-emerald-400">
            ●{healthyNodes} healthy
          </span>
          <span class="text-orange-400">
            ●{degradedNodes} degraded
          </span>
          <span class="text-red-400">
            ●{unhealthyNodes} unhealthy
          </span>
          <span class="text-zinc-500">
            avg: {formatResponseTime(avgResponseTime)}
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
        {#if !hasData}
          <!-- Loading/No Data State - Show Skeletons -->
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
                  </div>
                </div>
                <div class="mb-0.5">
                  <Skeleton class="w-32 h-2" />
                </div>
                <div class="flex items-center justify-between">
                  <div class="flex items-center space-x-2">
                    <Skeleton class="w-16 h-4" />
                    <Skeleton class="w-8 h-2" />
                    <Skeleton class="w-12 h-2" />
                  </div>
                </div>
              </div>
            {/each}
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
