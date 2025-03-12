<script lang="ts">
import { statistics, dailyTransfers } from "$lib/stores/statistics.svelte"
import { statisticsQuery, dailyTransfersQuery } from "$lib/queries/statistics.svelte"
import { Option } from "effect"
import { onMount } from "svelte"
import Card from "$lib/components/ui/Card.svelte"
import Label from "$lib/components/ui/Label.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import StatisticComponent from "$lib/components/model/StatisticComponent.svelte"
import BarChart from "$lib/components/model/BarChart.svelte"

onMount(() => {
  statistics.runEffect(statisticsQuery)
  dailyTransfers.runEffect(dailyTransfersQuery(30))

  return () => {
    statistics.interruptFiber()
    dailyTransfers.interruptFiber()
  }
})
</script>

<Sections>
  <!-- Statistics Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    {#if Option.isSome(statistics.data)}
      {#each statistics.data.value as stat}
        <StatisticComponent statistic={stat} />
      {/each}
    {:else if Option.isSome(statistics.error)}
      <div class="col-span-full">
        <ErrorComponent error={statistics.error.value} />
      </div>
    {:else}
      {#each Array(2) as _}
        <Card class="animate-pulse">
          <div class="h-4 bg-zinc-200 dark:bg-zinc-700 rounded w-3/4 mb-4"></div>
          <div class="h-8 bg-zinc-200 dark:bg-zinc-700 rounded w-1/2"></div>
        </Card>
      {/each}
    {/if}
  </div>
  
  <!-- Daily Transfers Chart -->
  <Card divided>
    <div class="p-4 gap-4 -mb-14 border-b-0">
      <h2 class="text-2xl font-bold mb-1">Daily Transfers</h2>
      <Label>Last 30 days of transfer activity</Label>
    </div>
    
    <BarChart data={dailyTransfers.data} error={dailyTransfers.error} />
  </Card>
 
</Sections>


