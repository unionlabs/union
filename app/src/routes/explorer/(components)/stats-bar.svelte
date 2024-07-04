<script lang="ts">
import StatsBarStat from "$lib/components/stats-bar-stat.svelte"
import { packetCountQuery, transferCountQuery, transfersPerDayQuery } from "$lib/queries/stats.ts"
import PixelGraph from "../(components)/pixel-graph.svelte"
import { userTime } from "$lib/utilities/user-time.ts"
import SpinningOutlineLogoThree from "$lib/components/spinning-outline-logo-three.svelte"
import { Separator } from "$lib/components/ui/separator"

// 30 days
$: transfersPerDayData = transfersPerDayQuery(30)

$: packetCountData = packetCountQuery()
$: transferCountData = transferCountQuery()
</script>

  <div class="bg-muted dark:bg-background border-b flex">
    <div class="w-full flex flex-1">
      <StatsBarStat label={"Total Transfers"} value={$transferCountData?.data?.aggregate?.count || 0} blink={true}/>
      <Separator orientation="vertical"/>
      <StatsBarStat label="Total Packets" value={$packetCountData?.data?.aggregate?.count || 0} blink={true}/>
      <Separator orientation="vertical"/>
      <StatsBarStat label="Metrics" value={$userTime} blink={false}>
        {#if $transfersPerDayData.data}
          <div class="ml-6 flex items-end">
            <PixelGraph data={$transfersPerDayData.data}/>
          </div>
        {/if}
      </StatsBarStat>
      <Separator orientation="vertical"/>
      <SpinningOutlineLogoThree/>
    </div>
  </div>
