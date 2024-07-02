<script lang="ts">
  import StatsBarStat from "$lib/components/stats-bar-stat.svelte";
  import { packetCountQuery, transferCountQuery, transfersPerDayQuery } from "$lib/queries/stats.ts";
  import PixelGraph from "../(components)/pixel-graph.svelte"
  import { userTime } from "$lib/utilities/user-time.ts";

  $: transfersPerDayData = transfersPerDayQuery()
  $: packetCountData = packetCountQuery()
  $: transferCountData= transferCountQuery()

  $: console.log($packetCountData)
  $: console.log($transferCountData)
</script>

<div class="bg-muted border-b flex divide-x">
  <StatsBarStat label={"Total Messages"} value={155_300_677}/>
  <StatsBarStat label="Total Packets" value={123_325_332} />

  <StatsBarStat label="Metrics" value={$userTime}>
    {#if $transfersPerDayData.data}
      <div class="ml-5 flex items-end">
        <PixelGraph data={$transfersPerDayData.data}/>
      </div>
    {/if}
  </StatsBarStat>
</div>
