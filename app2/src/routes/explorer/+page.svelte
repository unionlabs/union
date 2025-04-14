<script lang="ts">
  import {dailyTransfers, statistics} from "$lib/stores/statistics.svelte"
  import {dailyTransfersQuery, statisticsQuery} from "$lib/queries/statistics.svelte"
  import {Option} from "effect"
  import {onMount} from "svelte"
  import Card from "$lib/components/ui/Card.svelte"
  import Label from "$lib/components/ui/Label.svelte"
  import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
  import Sections from "$lib/components/ui/Sections.svelte"
  import StatisticComponent from "$lib/components/model/StatisticComponent.svelte"
  import BarChart from "$lib/components/model/BarChart.svelte"
  import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
  import type {DailyTransfer} from "@unionlabs/sdk/schema"

  // State for tracking the currently hovered day
let hoveredDay = $state<Option.Option<DailyTransfer>>(Option.none())

// Find the day with the highest count
const highestDay = $derived.by(() => {
  if (!Option.isSome(dailyTransfers.data) || dailyTransfers.data.value.length === 0)
    return Option.none()
  return Option.some(
    dailyTransfers.data.value.reduce(
      (max, current) => (current.count > max.count ? current : max),
      dailyTransfers.data.value[0]
    )
  )
})

// The count to display (either hovered day or highest day)
const displayDay = $derived(
  Option.isSome(hoveredDay)
    ? hoveredDay.value
    : Option.isSome(highestDay)
      ? highestDay.value
      : undefined
)

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
        <Card class="h-22 animate-pulse">
          <div></div>
        </Card>
      {/each}
    {/if}
  </div>

  <Card divided>
    <A class="block p-4" href="/explorer/transfers" external={false}>View all transfers</A>
    <A class="block p-4" href="/explorer/packets" external={false}>View all packets</A>
  </Card>
  
  <!-- Daily Transfers Chart -->
  <!--
  <Card class="h-80 relative" divided>
    <div class="p-4 gap-4 absolute top-0 left-0 border-b-0 w-full z-10">
      <div class="flex justify-between items-center">
        {#if displayDay !== undefined}
          <div>
            <Label>Transfers</Label>
            <div class="text-2xl font-bold mt-1">{displayDay.count.toLocaleString()}</div>
            {#if Option.isSome(hoveredDay)}<Label class="mt-1"><DateTimeComponent class="text-zinc-500" value={hoveredDay.value.day_date} showTime={false} /></Label>{/if}
          </div>
        {/if}
      </div>
    </div>
    
    <BarChart 
      data={dailyTransfers.data} 
      error={dailyTransfers.error} 
      onHoverChange={(day) => hoveredDay = day}
    />
  </Card>
  !-->
 
</Sections>


