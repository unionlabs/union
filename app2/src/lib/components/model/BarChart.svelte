<script lang="ts">
import { Option } from "effect"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import type { DailyTransfer } from "$lib/schema/statistics"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { onMount } from "svelte"

type Props = {
  data: Option.Option<ReadonlyArray<DailyTransfer>>
  error: Option.Option<FetchDecodeGraphqlError>
  class?: string
  onHoverChange?: (day: Option.Option<DailyTransfer>) => void
}

const { data, error, class: className = "", onHoverChange = () => {} }: Props = $props()

// Format large numbers with commas (used for chart tooltips)
function formatNumber(num: string | number): string {
  return Number(num).toLocaleString()
}

// Derived values for chart data
const reversedDailyTransfers = $derived(Option.isSome(data) ? [...data.value].reverse() : [])

const maxCount = $derived(Option.isSome(data) ? Math.max(...data.value.map(d => d.count)) : 0)

// Track the currently hovered day for display
let hoveredDay = $state<Option.Option<DailyTransfer>>(Option.none())

// Find the day with the highest count
const highestDay = $derived.by(() => {
  if (!Option.isSome(data) || data.value.length === 0) return Option.none()
  return Option.some(
    data.value.reduce((max, current) => (current.count > max.count ? current : max), data.value[0])
  )
})

// The count to display (either hovered day or highest day)
const displayCount = $derived(() =>
  Option.isSome(hoveredDay)
    ? hoveredDay.value.count
    : Option.isSome(highestDay)
      ? highestDay.value.count
      : 0
)
const displayDate = $derived(() =>
  Option.isSome(hoveredDay)
    ? hoveredDay.value.day
    : Option.isSome(highestDay)
      ? highestDay.value.day
      : ""
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
    heightPercent: maxCount > 0 ? Math.max((day.count / maxCount) * 100, 1) : 1
  }))
)

// Get labels for x-axis (first, middle, last)
const xAxisLabels = $derived(
  reversedDailyTransfers.filter(
    (_, i, arr) => i === 0 || i === Math.floor(arr.length / 2) || i === arr.length - 1
  )
)
</script>

{#if Option.isSome(data) && maxCount > 0}
  <!-- Chart container -->
  <div class="h-full relative chart-container {className}">
    <!-- Grid lines -->
    <div class="absolute left-0 right-0 top-0 bottom-0 flex flex-col justify-between">
      {#each Array(5) as _, i}
        <div class="border-t first:border-0 border-zinc-200 dark:border-zinc-900 w-full h-0"></div>
      {/each}
    </div>
    
    <!-- Bars -->
    <div class="absolute left-0 right-0 top-0 bottom-0 pt-1 px-4 pt-4">
      <div class="flex h-full items-end">
        {#each barHeights as day, i}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div 
            class="flex pr-1 flex-col flex-1 group size-full justify-end hover:opacity-100"
            onmouseenter={() => {
              hoveredDay = Option.some(day);
              onHoverChange(Option.some(day));
            }}
            onmouseleave={() => {
              hoveredDay = Option.none();
              onHoverChange(Option.none());
            }}
          >
            <div class="w-full size-full flex items-end">
              <div 
                class="relative w-full bg-white bar animate-bar"
                style="--final-height: {day.heightPercent}%; --delay: {i * 50}ms; min-height: 1px;"
              >
                <!-- uncomment for tooltip
                <div class="absolute pointer-events-none bottom-full mb-2 left-1/2 transform -translate-x-1/2 bg-zinc-950 border-zinc-900 border text-white dark:text-white px-2 py-1 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
                  <div>{formatNumber(day.count)}</div> <DateTimeComponent value={day.day} showTime={false} />
                </div>
                !-->
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{:else if Option.isSome(error)}
  <ErrorComponent error={error.value} />
{/if}

<style>
  /* Style for chart bars - make non-hovered bars darker when any bar is hovered */
  :global(.chart-container:hover .flex-1) {
    opacity: 0.3;
  }
  
  :global(.chart-container .flex-1:hover) {
    opacity: 1 !important;
  }

  /* Bar animation */
  .animate-bar {
    height: 0;
    animation: grow-bar 0.8s cubic-bezier(0.22, 1, 0.36, 1) forwards;
    animation-delay: var(--delay, 0ms);
  }

  @keyframes grow-bar {
    from {
      height: 0;
    }
    to {
      height: var(--final-height, 0%);
    }
  }
</style>
