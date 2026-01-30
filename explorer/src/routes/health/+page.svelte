<script lang="ts">
import CornerMarks from "$lib/components/corner-marks.svelte"
import { Badge } from "$lib/components/ui/badge/index.js"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { type ChainStatus, type HealthResponse, indexer } from "$lib/services/indexer-client"
import ActivityIcon from "@lucide/svelte/icons/activity"
import AlertCircleIcon from "@lucide/svelte/icons/alert-circle"
import CheckCircleIcon from "@lucide/svelte/icons/check-circle"
import DatabaseIcon from "@lucide/svelte/icons/database"
import LoaderIcon from "@lucide/svelte/icons/loader"
import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw"
import { onMount } from "svelte"

let health = $state<HealthResponse | null>(null)
let error = $state<string | null>(null)
let loading = $state(true)
let lastUpdated = $state<Date | null>(null)

async function fetchHealth() {
  loading = true
  error = null
  try {
    health = await indexer.health()
    lastUpdated = new Date()
  } catch (e) {
    error = String(e)
  } finally {
    loading = false
  }
}

onMount(() => {
  fetchHealth()
  const interval = setInterval(fetchHealth, 5000)
  return () => clearInterval(interval)
})

function getStatusColor(status: ChainStatus["status"]): string {
  switch (status) {
    case "synced":
      return "bg-green-500"
    case "syncing":
      return "bg-blue-500"
    case "backfilling":
      return "bg-yellow-500"
    case "error":
      return "bg-red-500"
    default:
      return "bg-gray-500"
  }
}

function getStatusVariant(
  status: ChainStatus["status"],
): "success" | "secondary" | "destructive" | "default" {
  switch (status) {
    case "synced":
      return "success"
    case "syncing":
    case "backfilling":
      return "default"
    case "error":
      return "destructive"
    default:
      return "secondary"
  }
}

function formatNumber(n: number): string {
  return n.toLocaleString()
}

function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  if (hours > 0) {
    return `${hours}h ${minutes}m ${secs}s`
  }
  if (minutes > 0) {
    return `${minutes}m ${secs}s`
  }
  return `${secs}s`
}

function formatBytes(bytes: number): string {
  if (bytes === 0) {
    return "0 B"
  }
  const units = ["B", "KB", "MB", "GB", "TB"]
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const value = bytes / Math.pow(1024, i)
  return `${value.toFixed(i > 0 ? 2 : 0)} ${units[i]}`
}
</script>

<svelte:head>
  <title>Indexer Health | Explorer</title>
</svelte:head>

<div class="p-6 max-w-6xl mx-auto space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold">Indexer Health</h1>
      <p class="text-sm text-muted-foreground mt-1">
        Monitor indexer sync status across all chains
      </p>
    </div>
    <button
      onclick={fetchHealth}
      disabled={loading}
      class="flex items-center gap-2 px-3 py-2 border border-border hover:bg-muted transition-colors disabled:opacity-50"
    >
      <RefreshCwIcon class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
      <span class="text-sm">Refresh</span>
    </button>
  </div>

  {#if error}
    <div class="relative border border-destructive/50 p-6">
      <CornerMarks />
      <div class="flex items-center gap-3">
        <AlertCircleIcon class="h-5 w-5 text-destructive" />
        <div>
          <p class="font-medium text-destructive">Failed to connect to indexer</p>
          <p class="text-sm text-muted-foreground mt-1 font-mono">{error}</p>
        </div>
      </div>
    </div>
  {:else if loading && !health}
    <div class="space-y-4">
      <Skeleton class="h-32" />
      <Skeleton class="h-64" />
    </div>
  {:else if health}
    <!-- Overall Status -->
    <div class="relative border border-border">
      <CornerMarks />
      <div class="p-6">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="p-3 {health.status === 'healthy' ? 'bg-green-500/20' : health.status === 'degraded' ? 'bg-red-500/20' : 'bg-yellow-500/20'}">
              {#if health.status === "healthy"}
                <CheckCircleIcon class="h-6 w-6 text-green-500" />
              {:else if health.status === "degraded"}
                <AlertCircleIcon class="h-6 w-6 text-red-500" />
              {:else}
                <LoaderIcon class="h-6 w-6 text-yellow-500 animate-spin" />
              {/if}
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-lg font-semibold capitalize">{health.status}</h2>
                <Badge
                  variant={health.status === "healthy"
                  ? "success"
                  : health.status === "degraded"
                  ? "destructive"
                  : "default"}
                >
                  {health.chains.length} chains
                </Badge>
              </div>
              <p class="text-sm text-muted-foreground">
                Started {new Date(health.started_at).toLocaleString()}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-8">
            <div class="text-right">
              <div class="text-2xl font-mono font-bold">{formatBytes(health.db_size_bytes)}</div>
              <div class="text-xs text-muted-foreground">database size</div>
            </div>
            <div class="text-right">
              <div class="text-2xl font-mono font-bold">{formatUptime(health.uptime_seconds)}</div>
              <div class="text-xs text-muted-foreground">uptime</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Chain Status Grid -->
    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {#each health.chains as chain}
        <div class="relative border border-border">
          <CornerMarks />

          <!-- Header -->
          <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full {getStatusColor(chain.status)}"></div>
              <span class="font-medium">{chain.name}</span>
            </div>
            <Badge variant={getStatusVariant(chain.status)}>
              {chain.status}
            </Badge>
          </div>

          <!-- Stats -->
          <div class="p-4 space-y-3">
            <!-- Progress bar for backfill -->
            {#if chain.status === "backfilling"}
              <div>
                <div class="flex justify-between text-xs mb-1">
                  <span class="text-muted-foreground">Backfill Progress</span>
                  <span class="font-mono">{chain.backfillProgress}%</span>
                </div>
                <div class="h-2 bg-muted overflow-hidden">
                  <div
                    class="h-full bg-yellow-500 transition-all duration-500"
                    style="width: {chain.backfillProgress}%"
                  >
                  </div>
                </div>
              </div>
            {/if}

            <div class="grid grid-cols-2 gap-3 text-sm">
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">
                  Latest Height
                </div>
                <div class="font-mono">{formatNumber(chain.latestHeight)}</div>
              </div>
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">
                  Indexed Height
                </div>
                <div class="font-mono">{formatNumber(chain.indexedHeight)}</div>
              </div>
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">Blocks</div>
                <div class="font-mono">{formatNumber(chain.blocksIndexed)}</div>
              </div>
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">
                  Transactions
                </div>
                <div class="font-mono">{formatNumber(chain.txsIndexed)}</div>
              </div>
            </div>

            {#if chain.lastSync}
              <div class="text-xs text-muted-foreground pt-2 border-t border-border">
                Last sync: {new Date(chain.lastSync).toLocaleTimeString()}
              </div>
            {/if}

            {#if chain.lastError}
              <div
                class="text-xs text-destructive pt-2 border-t border-border font-mono truncate"
                title={chain.lastError}
              >
                {chain.lastError}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Footer -->
    {#if lastUpdated}
      <div class="text-center text-xs text-muted-foreground">
        Last updated: {lastUpdated.toLocaleTimeString()} · Auto-refreshes every 5s
      </div>
    {/if}
  {/if}
</div>
