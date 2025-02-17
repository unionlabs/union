<script lang="ts">
import { onDestroy } from "svelte"
import LoadingDots from "$lib/components/loading-dots.svelte"
import { createQuery } from "@tanstack/svelte-query"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { raise } from "$lib/utilities"
import { OrderStatsDocument } from "$lib/graphql/queries/stats.ts"
import { showDetailedTrace } from "$lib/stores/user.ts"

export let transferStatus: "acknowledged" | "transferred" | "transferring"
export let sourceChainId: string
export let destinationChainId: string
export let sentTimestamp: string

// Formats a number of seconds into a human-readable string
const formatSeconds = (seconds: number | undefined) => {
  if (seconds === undefined || Number.isNaN(seconds)) return "N/A"
  if (seconds < 60) return `${seconds.toFixed(0)}s`
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const remainingSeconds = seconds % 60
  let result = ""
  if (hours > 0) result += `${hours}h`
  if (minutes > 0) result += `${minutes}m`
  if (remainingSeconds > 0) result += `${remainingSeconds.toFixed(0)}s`
  return result
}

// Query returns only the daily stats (interval_secs === 86400)
let stats = createQuery({
  queryKey: ["order-stats", sourceChainId, destinationChainId],
  refetchInterval: 60000,
  placeholderData: previousData => previousData,
  queryFn: async () => {
    const response = await request(URLS().GRAPHQL, OrderStatsDocument, {
      sourceChainId,
      destinationChainId
    })
    if (!response?.v1_ibc_union_fungible_asset_order_stats) {
      raise("Error fetching order stats")
    }
    // Return the daily stats object (interval_secs === 86400)
    return response.v1_ibc_union_fungible_asset_order_stats[0]
  }
})

$: progressMap = $showDetailedTrace
  ? {
      transferring: 33,
      transferred: 66,
      acknowledged: 100
    }
  : {
      transferring: 50,
      transferred: 100
    }

// Display texts
$: statusText = {
  transferring: "Your funds are on the way. Please hold tight.",
  transferred: `This transfer is complete. Your funds are ready to use on ${$stats.data?.destination_chain?.display_name}`,
  acknowledged: "Transfer complete. Your funds are confirmed."
}

const statusTitle = {
  transferring: "TRANSFER IN PROGRESS",
  transferred: "TRANSFER COMPLETE",
  acknowledged: "TRANSFER CONFIRMED"
}

$: progress = progressMap[transferStatus]
$: text = statusText[transferStatus]
$: title = statusTitle[transferStatus]

// Reactive declarations for ETA calculations using daily stats
$: medianRecv = $stats.data?.secs_until_packet_recv?.median
$: worstRecv = $stats.data?.secs_until_packet_recv?.p95
$: medianAck = $stats.data?.secs_until_packet_ack?.median
$: worstAck = $stats.data?.secs_until_packet_ack?.p95

$: effectiveStart = new Date(sentTimestamp).getTime()

// Set up a reactive current time updated every second
let now = Date.now()
const interval = setInterval(() => {
  now = Date.now()
}, 1000)
onDestroy(() => clearInterval(interval))

// Calculate elapsed time in seconds since effectiveStart
$: elapsed = (now - effectiveStart) / 1000
// Remaining time = median - elapsed, but not less than 0
$: remainingRecv = medianRecv ? Math.max(0, medianRecv - elapsed) : undefined
$: remainingAck = medianAck ? Math.max(0, medianAck - elapsed) : undefined
// Delay = elapsed - median, if elapsed exceeds median
$: delayRecv = medianRecv ? Math.max(0, elapsed - medianRecv) : 0
$: delayAck = medianAck ? Math.max(0, elapsed - medianAck) : 0
</script>

{#if !$stats.isError}
  <div class="border-2 p-4 space-y-4 w-full">
    <h2 class="text-lg font-bold">{title}</h2>

    <!-- PROGRESS LINE -->
    <div class="w-full">
      <div class="relative h-4 bg-gray-200 overflow-hidden border border-2 border-black">
        <!-- Vertical markers at 33% and 66% -->
        {#if $showDetailedTrace}
        <div class="absolute left-[33%] top-0 h-full w-[2px] bg-black"></div>
        <div class="absolute left-[66%] top-0 h-full w-[2px] bg-black"></div>
          {:else}
          <div class="absolute left-[50%] top-0 h-full w-[2px] bg-black"></div>
        {/if}
        <!-- Progress bar fill -->
        <div class="h-full bg-accent transition-all duration-300" style="width: {progress}%;"></div>
      </div>
      <div class="mt-4 text-sm font-medium">{text}</div>
    </div>

    <!-- MAIN CONTENT AREA -->
    {#if transferStatus === "transferring" || !showDetailedTrace}
      <div>
        {#if $stats.isLoading || !medianRecv}
          <p class="mt-1 text-neutral-400 text-sm">Calculating ETA...</p>
        {:else}
          {#if delayRecv > 0.2 * medianRecv}
            <p class="mt-1 text-sm">
              Your transfer is taking {formatSeconds(delayRecv)} longer than the average transfer.
              The slowest transfer for this connection in the past day was {formatSeconds(worstRecv)}.
            </p>
          {:else}
            <p class="mt-1 text-sm">
              On average, this transfer will be completed in {formatSeconds(remainingRecv)} from now.
            </p>
          {/if}
        {/if}
      </div>
    {:else if transferStatus === "transferred" && $showDetailedTrace}
      <div>
        <h3 class="text-xs font-semibold">Acknowledgement ETA (for developers):</h3>
        {#if $stats.isLoading || !medianAck}
          <p class="mt-1 text-neutral-400 text-xs">Calculating ACK ETA...</p>
        {:else}
          {#if delayAck > 0.2 * medianAck}
            <p class="mt-1 text-xs">
              Your transfer is taking {formatSeconds(delayAck)} longer than the average acknowledgment.
              The slowest acknowledgment for this connection in the past day was {formatSeconds(worstAck)}.
            </p>
          {:else}
            <p class="mt-1 text-xs">
              On average, this transfer will be acknowledged in {formatSeconds(remainingAck)} from now.
            </p>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
{/if}
