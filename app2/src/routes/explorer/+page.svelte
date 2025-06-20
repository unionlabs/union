<script lang="ts">
import BarChart from "$lib/components/model/BarChart.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import StatisticComponent from "$lib/components/model/StatisticComponent.svelte"
import A from "$lib/components/ui/A.svelte"
import Card from "$lib/components/ui/Card.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { dailyPacketsQuery, dailyTransfersQuery, statisticsQuery } from "$lib/queries/statistics.svelte"
import { dailyPackets, dailyTransfers, statistics } from "$lib/stores/statistics.svelte"
import type { DailyTransfer } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import { onMount } from "svelte"

// State for tracking the currently hovered day
let hoveredTransferDay = $state<Option.Option<DailyTransfer>>(Option.none())
let hoveredPacketDay = $state<Option.Option<DailyTransfer>>(Option.none())

// Find the day with the highest count for transfers
const highestTransferDay = $derived.by(() => {
  if (!Option.isSome(dailyTransfers.data) || dailyTransfers.data.value.length === 0) {
    return Option.none()
  }
  return Option.some(
    dailyTransfers.data.value.reduce(
      (max, current) => (current.count > max.count ? current : max),
      dailyTransfers.data.value[0],
    ),
  )
})

// Find the day with the highest count for packets
const highestPacketDay = $derived.by(() => {
  if (!Option.isSome(dailyPackets.data) || dailyPackets.data.value.length === 0) {
    return Option.none()
  }
  return Option.some(
    dailyPackets.data.value.reduce(
      (max, current) => (current.count > max.count ? current : max),
      dailyPackets.data.value[0],
    ),
  )
})

// The count to display for transfers (either hovered day or highest day)
const displayTransferDay = $derived(
  Option.isSome(hoveredTransferDay)
    ? hoveredTransferDay.value
    : Option.isSome(highestTransferDay)
    ? highestTransferDay.value
    : undefined,
)

// The count to display for packets (either hovered day or highest day)
const displayPacketDay = $derived(
  Option.isSome(hoveredPacketDay)
    ? hoveredPacketDay.value
    : Option.isSome(highestPacketDay)
    ? highestPacketDay.value
    : undefined,
)

onMount(() => {
  statistics.runEffect(statisticsQuery)
  dailyTransfers.runEffect(dailyTransfersQuery())
  dailyPackets.runEffect(dailyPacketsQuery())

  return () => {
    statistics.interruptFiber()
    dailyTransfers.interruptFiber()
    dailyPackets.interruptFiber()
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

  <!-- Daily Transfers Chart -->
  <Card
    class="h-80 relative"
    divided
  >
    <div class="p-4 gap-4 absolute top-0 left-0 border-b-0 w-full z-10">
      <div class="flex justify-between items-center">
        {#if displayTransferDay !== undefined}
          <div>
            <Label>Transfers</Label>
            <div class="text-2xl font-bold mt-1">{displayTransferDay.count.toLocaleString()}</div>
            {#if Option.isSome(hoveredTransferDay)}<Label class="mt-1"><DateTimeComponent
                  class="text-zinc-500"
                  value={hoveredTransferDay.value.day_date}
                  showTime={false}
                /></Label>{/if}
          </div>
        {/if}
      </div>
    </div>

    <BarChart
      data={dailyTransfers.data}
      error={dailyTransfers.error}
      onHoverChange={(day) => hoveredTransferDay = day}
    />
  </Card>

  <!-- Daily Packets Chart -->
  <Card
    class="h-80 relative"
    divided
  >
    <div class="p-4 gap-4 absolute top-0 left-0 border-b-0 w-full z-10">
      <div class="flex justify-between items-center">
        {#if displayPacketDay !== undefined}
          <div>
            <Label>Packets</Label>
            <div class="text-2xl font-bold mt-1">{displayPacketDay.count.toLocaleString()}</div>
            {#if Option.isSome(hoveredPacketDay)}<Label class="mt-1"><DateTimeComponent
                  class="text-zinc-500"
                  value={hoveredPacketDay.value.day_date}
                  showTime={false}
                /></Label>{/if}
          </div>
        {/if}
      </div>
    </div>

    <BarChart
      data={dailyPackets.data}
      error={dailyPackets.error}
      onHoverChange={(day) => hoveredPacketDay = day}
    />
  </Card>

  <Card divided>
    <A
      class="block p-4"
      href="/explorer/transfers"
      external={false}
    >View all transfers</A>
    <A
      class="block p-4"
      href="/explorer/packets"
      external={false}
    >View all packets</A>
  </Card>
</Sections>
