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

const yLabels = $derived([
  0,
  Math.round(maxCount / 4),
  Math.round(maxCount / 2),
  Math.round((maxCount * 3) / 4),
  maxCount
])

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
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
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
  <Card>
    <div class="mb-6">
      <h2 class="text-xl font-bold">Daily Transfers</h2>
      <Label>Last 30 days of transfer activity</Label>
    </div>
    
    {#if Option.isSome(dailyTransfers.data)}
      <div class="overflow-x-auto">
        <!-- Chart container -->
        <div class="h-64 relative" style="min-height: 16rem;">
          <!-- Y-axis labels -->
          <div class="absolute left-0 top-0 bottom-0 w-12 flex flex-col justify-between text-xs text-zinc-500 dark:text-zinc-400 pr-2">
            {#each yLabels as label, i}
              <div class="text-right" style="transform: translateY({i === 0 ? '100%' : i === yLabels.length - 1 ? '0' : 'none'})">
                {formatNumber(label)}
              </div>
            {/each}
          </div>
          
          <!-- Grid lines -->
          <div class="absolute left-12 right-0 top-0 bottom-0 flex flex-col justify-between">
            {#each Array(5) as _, i}
              <div class="border-t border-zinc-200 dark:border-zinc-700 w-full h-0"></div>
            {/each}
          </div>
          
          <!-- Bars -->
          <div class="absolute left-12 right-0 top-0 bottom-0 pt-1 pb-6">
            <div class="flex h-full items-end" style="min-height: 12rem;">
              {#each barHeights as day, i}
                <div class="flex flex-col items-center flex-1 group size-full">
                  <div class="relative w-full px-1 size-full">
                    <div 
                      class="w-full bg-blue-500 dark:bg-blue-400 rounded-t transition-all duration-300 group-hover:bg-blue-600 dark:group-hover:bg-blue-300"
                      style="height: {day.heightPercent}%; min-height: 1px;"
                    >
                      <div class="absolute bottom-full mb-1 left-1/2 transform -translate-x-1/2 bg-zinc-800 dark:bg-zinc-700 text-white dark:text-white px-2 py-1 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
                        {formatNumber(day.count)} transfers on <DateTimeComponent value={day.day} showSeconds={false} />
                      </div>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
          
          <!-- X-axis labels -->
          <div class="absolute left-12 right-0 bottom-0 flex justify-between text-xs text-zinc-500 dark:text-zinc-400">
            {#each xAxisLabels as day}
              <div class="text-center px-2 whitespace-nowrap">
                <DateTimeComponent value={day.day} showTime={false} />
              </div>
            {/each}
          </div>
        </div>
      </div>
      
      <div class="mt-4 text-center text-sm text-zinc-500 dark:text-zinc-400">
        Showing the last 30 days of transfer activity. View the table below for complete data.
      </div>
    {:else if Option.isSome(dailyTransfers.error)}
      <ErrorComponent error={dailyTransfers.error.value} />
    {:else}
      <div class="h-64 animate-pulse bg-zinc-200 dark:bg-zinc-700 rounded"></div>
    {/if}
  </Card>
  
  <!-- Data Table -->
  {#if Option.isSome(dailyTransfers.data)}
    <Card class="overflow-hidden p-0">
      <table class="min-w-full divide-y divide-zinc-200 dark:divide-zinc-700">
        <thead class="bg-zinc-50 dark:bg-zinc-900">
          <tr>
            <th class="px-6 py-3 text-left">
              <Label>Date</Label>
            </th>
            <th class="px-6 py-3 text-left">
              <Label>Transfer Count</Label>
            </th>
          </tr>
        </thead>
        <tbody class="bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700">
          {#each dailyTransfers.data.value as day}
            <tr class="hover:bg-zinc-50 dark:hover:bg-zinc-700/50 transition-colors">
              <td class="px-6 py-4 whitespace-nowrap text-sm text-zinc-900 dark:text-zinc-100">
                <DateTimeComponent value={day.day} showTime={false} />
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-zinc-900 dark:text-zinc-100">{formatNumber(day.count)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </Card>
  {/if}
</Sections>
