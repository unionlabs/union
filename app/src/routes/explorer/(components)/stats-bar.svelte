<script lang="ts">
  import StatsBarStat from "$lib/components/stats-bar-stat.svelte";
  import { transferPerDayQuery } from "$lib/queries/stats.ts";
  import PixelGraph from "../(components)/pixel-graph.svelte"

  $: trasferPerDayData = transferPerDayQuery()
</script>

<div class="bg-muted border-b flex divide-x">
  <StatsBarStat label={"Total Messages"} value={155_300_677}/>
  {#if $trasferPerDayData.data}
    <PixelGraph data={$trasferPerDayData.data}/>
  {/if}
</div>
