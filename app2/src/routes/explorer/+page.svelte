<script lang="ts">
import { statistics, dailyTransfers } from "$lib/stores/statistics.svelte"
import { statisticsQuery, dailyTransfersQuery } from "$lib/queries/statistics.svelte"
import { Option } from "effect"
import { cn } from "$lib/utils"
import { onMount } from "svelte"

onMount(() => {
  statistics.runEffect(statisticsQuery)
  dailyTransfers.runEffect(dailyTransfersQuery(30))

  return () => {
    statistics.interruptFiber()
    dailyTransfers.interruptFiber()
  }
})

// Format large numbers with commas
function formatNumber(num: string | number): string {
  return Number(num).toLocaleString()
}

// Format date strings
function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString()
}
</script>

<div class="container mx-auto px-4 py-8">
  <!-- Statistics Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
    {#if Option.isSome(statistics.data)}
      {#each statistics.data.value as stat}
        <div class="bg-white dark:bg-zinc-800 rounded-lg shadow-md p-6 transition-all hover:shadow-lg">
          <h3 class="text-zinc-500 dark:text-zinc-400 text-sm font-medium mb-2">{stat.name}</h3>
          <p class="text-2xl font-bold">{formatNumber(stat.value)}</p>
        </div>
      {/each}
    {:else if Option.isSome(statistics.error)}
      <div class="col-span-full bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-4 rounded-lg">
        <p>Error loading statistics: {statistics.error.value.message}</p>
      </div>
    {:else}
      {#each Array(4) as _}
        <div class="bg-white dark:bg-zinc-800 rounded-lg shadow-md p-6 animate-pulse">
          <div class="h-4 bg-zinc-200 dark:bg-zinc-700 rounded w-3/4 mb-4"></div>
          <div class="h-8 bg-zinc-200 dark:bg-zinc-700 rounded w-1/2"></div>
        </div>
      {/each}
    {/if}
  </div>
  
  <!-- Daily Transfers Chart -->
  <div class="bg-white dark:bg-zinc-800 rounded-lg shadow-md p-6 mb-12">
    <h2 class="text-xl font-bold mb-6">Daily Transfers</h2>
    
    {#if Option.isSome(dailyTransfers.data)}
      <div class="overflow-x-auto">
        <!-- Chart container -->
        <div class="h-64 min-w-[600px]">
          <div class="flex h-full items-end">
            {#each [...dailyTransfers.data.value].reverse() as day, i}
              {@const maxCount = Math.max(...dailyTransfers.data.value.map(d => d.count))}
              {@const height = (day.count / maxCount) * 100}
              
              <div class="flex flex-col items-center flex-1 group">
                <div class="relative w-full px-1">
                  <div 
                    class="w-full bg-blue-500 dark:bg-blue-600 rounded-t transition-all duration-300 group-hover:bg-blue-600 dark:group-hover:bg-blue-500"
                    style="height: {Math.max(height, 1)}%"
                  >
                    <div class="absolute bottom-full mb-1 left-1/2 transform -translate-x-1/2 bg-zinc-800 dark:bg-zinc-700 text-white px-2 py-1 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap">
                      {formatNumber(day.count)} transfers on {formatDate(day.day)}
                    </div>
                  </div>
                </div>
                {#if i % 5 === 0 || i === dailyTransfers.data.value.length - 1}
                  <div class="text-xs text-zinc-500 dark:text-zinc-400 mt-2 rotate-45 origin-left">
                    {formatDate(day.day)}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      </div>
    {:else if Option.isSome(dailyTransfers.error)}
      <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-4 rounded-lg">
        <p>Error loading daily transfers: {dailyTransfers.error.value.message}</p>
      </div>
    {:else}
      <div class="h-64 animate-pulse bg-zinc-200 dark:bg-zinc-700 rounded"></div>
    {/if}
  </div>
  
  <!-- Data Table -->
  {#if Option.isSome(dailyTransfers.data)}
    <div class="bg-white dark:bg-zinc-800 rounded-lg shadow-md overflow-hidden">
      <table class="min-w-full divide-y divide-zinc-200 dark:divide-zinc-700">
        <thead class="bg-zinc-50 dark:bg-zinc-900">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider">Date</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider">Transfer Count</th>
          </tr>
        </thead>
        <tbody class="bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700">
          {#each dailyTransfers.data.value as day}
            <tr class="hover:bg-zinc-50 dark:hover:bg-zinc-700/50 transition-colors">
              <td class="px-6 py-4 whitespace-nowrap text-sm text-zinc-900 dark:text-zinc-100">{formatDate(day.day)}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-zinc-900 dark:text-zinc-100">{formatNumber(day.count)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
