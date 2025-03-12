<script lang="ts">
import { statistics, dailyTransfers } from "$lib/stores/statistics.svelte"
import { statisticsQuery, dailyTransfersQuery } from "$lib/queries/statistics.svelte"
import { Option, DateTime } from "effect"
import { cn } from "$lib/utils"
import { onMount } from "svelte"
import Card from "$lib/components/ui/Card.svelte"
import Label from "$lib/components/ui/Label.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import StatisticComponent from "$lib/components/model/StatisticComponent.svelte"

onMount(() => {
  statistics.runEffect(statisticsQuery)
  dailyTransfers.runEffect(dailyTransfersQuery(30))

  return () => {
    statistics.interruptFiber()
    dailyTransfers.interruptFiber()
  }
})

// Format large numbers with commas (used for chart tooltips)
function formatNumber(num: string | number): string {
  return Number(num).toLocaleString()
}

// Derived values for chart data
const reversedDailyTransfers = $derived(
  Option.isSome(dailyTransfers.data) ? [...dailyTransfers.data.value].reverse() : []
)

const maxCount = $derived(
  Option.isSome(dailyTransfers.data) ? Math.max(...dailyTransfers.data.value.map(d => d.count)) : 0
)

// Calculate nice round numbers for y-axis labels
const yLabels = $derived(() => {
  if (maxCount <= 0) return [0, 0, 0, 0, 0]

  // Find a nice round maximum that's at least as large as maxCount
  const magnitude = 10 ** Math.floor(Math.log10(maxCount))
  const roundedMax = Math.ceil(maxCount / magnitude) * magnitude

  // Create evenly spaced labels
  return [
    0,
    Math.round(roundedMax / 4),
    Math.round(roundedMax / 2),
    Math.round((roundedMax * 3) / 4),
    roundedMax
  ]
})

// Calculate bar heights as percentages
const barHeights = $derived(
  reversedDailyTransfers.map(day => ({
    ...day,
    heightPercent: Math.max((day.count / maxCount) * 100, 1)
  }))
)

// Get labels for x-axis (first, middle, last)
const xAxisLabels = $derived(
  reversedDailyTransfers.filter(
    (_, i, arr) => i === 0 || i === Math.floor(arr.length / 2) || i === arr.length - 1
  )
)
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
    
    {#if Option.isSome(dailyTransfers.data)}
        <!-- Chart container -->
        <div class="h-80 relative">
          <!-- Grid lines -->
          <div class="absolute left-0 right-0 top-0 bottom-0 flex flex-col justify-between">
            {#each Array(5) as _, i}
              <div class="border-t first:border-0 border-zinc-200 dark:border-zinc-900 w-full h-0"></div>
            {/each}
          </div>
          
          <!-- Bars -->
          <div class="absolute left-0 right-0 top-0 bottom-0 pt-1 px-4">
            <div class="flex h-full gap-[1px] sm:gap-[2px] md:gap-1 items-end" style="min-height: 12rem;">
              {#each barHeights as day, i}
                <div class="flex flex-col flex-1 group size-full justify-end">
                  <div class=" w-full size-full flex items-end">
                    <div 
                      class="relative w-full bg-white rounded-t transition-all duration-300 group-hover:bg-blue-600 dark:group-hover:bg-blue-300"
                      style="height: {day.heightPercent}%; min-height: 1px;"
                    >
                      <div class="absolute pointer-events-none bottom-full mb-2 left-1/2 transform -translate-x-1/2 bg-zinc-800 dark:bg-zinc-700 text-white dark:text-white px-2 py-1 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
                        {formatNumber(day.count)} on <DateTimeComponent value={day.day} showTime={false} />
                      </div>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
          
        </div>
    {:else if Option.isSome(dailyTransfers.error)}
      <ErrorComponent error={dailyTransfers.error.value} />
    {:else}
      <div class="h-64 animate-pulse bg-zinc-200 dark:bg-zinc-700 rounded"></div>
    {/if}
  </Card>
  
</Sections>
