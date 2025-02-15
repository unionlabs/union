<script lang="ts">
import LoadingDots from "$lib/components/loading-dots.svelte"
import { createQuery } from "@tanstack/svelte-query"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { raise } from "$lib/utilities"
import { OrderStatsDocument } from "$lib/graphql/queries/stats.ts"

export let transferStatus: "acknowledged" | "transferred" | "transferring"
export let sourceChainId: string
export let destinationChainId: string

const formatSeconds = (seconds: number | undefined) => {
  if (seconds === undefined) return undefined
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

let stats = createQuery({
  queryKey: ["order-stats", sourceChainId, destinationChainId],
  refetchInterval: 5000,
  placeholderData: previousData => previousData,
  queryFn: async () => {
    const response = await request(URLS().GRAPHQL, OrderStatsDocument, {
      sourceChainId,
      destinationChainId
    })
    if (!response?.v1_ibc_union_fungible_asset_order_stats) {
      raise("Error fetching order stats")
    }
    return response.v1_ibc_union_fungible_asset_order_stats[0]
  }
})

// Map transferStatus to progress percentages in 33% steps:
const progressMap = {
  transferring: 33,
  transferred: 66,
  acknowledged: 100
}

// Map statuses to corresponding display texts:
const statusText = {
  transferring: "Your funds are on the way. Please hold tight.",
  transferred: "Your funds are now available, pending final confirmation.",
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
</script>

{#if !$stats.isError}
  <div class="border border-2 p-4 space-y-4 w-full">

    <h2 class="text-lg font-bold">{title}</h2>

    <!-- PROGRESS LINE -->
    <div class="w-full">
      <div class="relative h-4 bg-gray-200 overflow-hidden border border-2 border-black">
        <!-- Vertical markers at 33% and 66% -->
        <div class="absolute left-[33%] top-0 h-full w-[2px] bg-black"></div>
        <div class="absolute left-[66%] top-0 h-full w-[2px] bg-black"></div>
        <!-- Progress bar fill -->
        <div
                class="h-full bg-accent transition-all duration-300"
                style="width: {progress}%;"
        ></div>
      </div>
      <div class="mt-4 text-sm font-medium">
        {text}
      </div>
    </div>

    <!-- HEADER & MAIN CONTENT AREA -->
    {#if transferStatus === "transferring"}
      <div>
        {#if $stats.isLoading}
          <div class="grid grid-cols-[auto_1fr] gap-2 text-neutral-400 mt-1">
            <span>Best:</span>
            <LoadingDots class="h-4 w-4" />
            <span>Avg:</span>
            <LoadingDots class="h-4 w-4" />
            <span>Worst:</span>
            <LoadingDots class="h-4 w-4" />
          </div>
        {:else}
          <div class="grid grid-cols-[auto_1fr] gap-2 text-muted-foreground mt-1">
            <span>Best:</span>
            <span class="font-bold">{formatSeconds($stats.data?.secs_until_packet_recv?.p5)}</span>
            <span>Avg:</span>
            <span class="font-bold">{formatSeconds($stats.data?.secs_until_packet_recv?.median)}</span>
            <span>Worst:</span>
            <span class="font-bold">{formatSeconds($stats.data?.secs_until_packet_recv?.p95)}</span>
          </div>
        {/if}
      </div>
    {:else if transferStatus === "transferred"}
      <div>
        <h3 class="text-sm font-semibold">ACK ETA:</h3>
        {#if $stats.isLoading || !$stats.data?.secs_until_packet_ack}
          <div class="grid grid-cols-[auto_1fr] gap-2 text-neutral-400 mt-1">
            <span>Best:</span>
            <LoadingDots class="h-4 w-4" />
            <span>Avg:</span>
            <LoadingDots class="h-4 w-4" />
            <span>Worst:</span>
            <LoadingDots class="h-4 w-4" />
          </div>
        {:else}
          <div class="grid grid-cols-[auto_1fr] gap-2 text-muted-foreground mt-1">
            <span>Best:</span>
            <span class="font-bold">{formatSeconds($stats.data.secs_until_packet_ack.p5)}</span>
            <span>Avg:</span>
            <span class="font-bold">{formatSeconds($stats.data.secs_until_packet_ack.median)}</span>
            <span>Worst:</span>
            <span class="font-bold">{formatSeconds($stats.data.secs_until_packet_ack.p95)}</span>
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}
