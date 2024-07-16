<script lang="ts">
import StatsBarStat from "$lib/components/stats-bar-stat.svelte"
import { packetCountQuery, transferCountQuery, transfersPerDayQuery } from "$lib/queries/stats.ts"
import PixelGraph from "../(components)/pixel-graph.svelte"
import { userTime } from "$lib/utilities/user-time.ts"
import SpinningOutlineLogo from "$lib/components/spinning-outline-logo.svelte"
import { onMount } from "svelte"
import { fade } from "svelte/transition"
import { Separator } from "$lib/components/ui/separator"

// 30 days
$: transfersPerDayData = transfersPerDayQuery(30)

$: packetCountData = packetCountQuery()
$: transferCountData = transferCountQuery()

let show = 1
let interval: any

onMount(() => {
  interval = setInterval(() => {
    show = show === 1 ? 2 : 1
  }, 5000)
  return () => {
    clearInterval(interval)
  }
})
</script>

<div class="bg-muted border-b flex dark:bg-background">
  <div class="w-full">
    {#if show === 1}
      <div class="w-full flex h-full" in:fade>
        <StatsBarStat label={"Total Transfers"} value={$transferCountData?.data?.aggregate?.count || 0}/>
        <Separator orientation="vertical"/>
        <StatsBarStat label="Total Packets" value={$packetCountData?.data?.aggregate?.count || 0}/>
      </div>
    {:else if show === 2}
      <div class="w-full flex" in:fade>
      <StatsBarStat label="Metrics" value={$userTime} on:click={() => show--}>
        {#if $transfersPerDayData.data}
          <div class="ml-6 flex items-end">
            <PixelGraph data={$transfersPerDayData.data}/>
          </div>
        {/if}
      </StatsBarStat>
      </div>
    {/if}
  </div>
</div>
