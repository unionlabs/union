<script lang="ts">
// Latency waterfall chart with box plots
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { Option, pipe } from "effect"

interface Props {
  latencyData: Option.Option<any[]>
}

let { latencyData }: Props = $props()

let selectedSource: string = $state("")
let selectedDestination: string = $state("")

// Extract unique chains for filters using Effect Option patterns
const allSources: string[] = $derived(pipe(
  latencyData,
  Option.map((data: any[]) =>
    data.length > 0
      ? [...new Set(data.map(item => item.sourceName).filter(Boolean))].sort()
      : []
  ),
  Option.getOrElse((): string[] => []),
))

const allDestinations: string[] = $derived(pipe(
  latencyData,
  Option.map((data: any[]) =>
    data.length > 0
      ? [...new Set(data.map(item => item.destinationName).filter(Boolean))].sort()
      : []
  ),
  Option.getOrElse((): string[] => []),
))

// Filter destinations based on selected source
const availableDestinations: string[] = $derived(pipe(
  latencyData,
  Option.map((data: any[]) => {
    if (data.length === 0) {
      return [] as string[]
    }
    if (!selectedSource) {
      return allDestinations
    }
    return [
      ...new Set(
        data
          .filter(item => item.sourceName === selectedSource)
          .map(item => item.destinationName)
          .filter(Boolean),
      ),
    ].sort()
  }),
  Option.getOrElse((): string[] => []),
))

const currentData = $derived(pipe(
  latencyData,
  Option.map(data => {
    if (data.length === 0) {
      return []
    }

    return data
      .filter(item =>
        item.packetRecv && item.writeAck && item.packetAck
        && item.packetRecv.median !== undefined
        && item.writeAck.median !== undefined
        && item.packetAck.median !== undefined
      )
      .filter(item => !selectedSource || item.sourceName === selectedSource)
      .filter(item => !selectedDestination || item.destinationName === selectedDestination)
      .map(item => {
        const eventTypes = [
          {
            key: "packetRecv",
            label: "Packet Recv",
            color: "bg-blue-600",
            shortLabel: "recv",
            p5: item.packetRecv.p5,
            median: item.packetRecv.median,
            p95: item.packetRecv.p95,
          },
          {
            key: "writeAck",
            label: "Write Ack",
            color: "bg-yellow-600",
            shortLabel: "write",
            p5: item.writeAck.p5,
            median: item.writeAck.median,
            p95: item.writeAck.p95,
          },
          {
            key: "packetAck",
            label: "Packet Ack",
            color: "bg-green-600",
            shortLabel: "ack",
            p5: item.packetAck.p5,
            median: item.packetAck.median,
            p95: item.packetAck.p95,
          },
        ]

        return {
          ...item,
          eventTypes,
        }
      })
      .sort((a, b) => a.packetAck.median - b.packetAck.median)
      .slice(0, !selectedSource && !selectedDestination ? 4 : 12)
  }),
  Option.getOrElse(() => []),
))

const hasData = $derived(currentData.length > 0)
const isLoading = $derived(
  !hasData && Option.isNone(latencyData),
)

// Calculate global scale across all visible routes for consistent scrubbing
const globalScale = $derived(
  !currentData || currentData.length === 0 ? { min: 0, max: 1 } : (() => {
    const allValues = currentData.flatMap(item => [
      item.packetRecv.p5,
      item.packetRecv.median,
      item.packetRecv.p95,
      item.writeAck.p5,
      item.writeAck.median,
      item.writeAck.p95,
      item.packetAck.p5,
      item.packetAck.median,
      item.packetAck.p95,
    ]).filter(v => v != null && !isNaN(v))

    if (allValues.length === 0) {
      return { min: 0, max: 1 }
    }

    const maxValue = Math.max(...allValues)
    const minValue = Math.min(...allValues)
    const padding = Math.max((maxValue - minValue) * 0.05, maxValue * 0.01)

    return {
      min: Math.max(0, minValue - padding),
      max: maxValue + padding,
    }
  })(),
)

function formatLatency(seconds: number): string {
  if (seconds < 1) {
    return `${(seconds * 1000).toFixed(0)}ms`
  }
  if (seconds < 10) {
    return `${seconds.toFixed(1)}s`
  }
  if (seconds < 60) {
    return `${seconds.toFixed(0)}s`
  }
  if (seconds < 3600) {
    return `${(seconds / 60).toFixed(1)}min`
  }
  return `${(seconds / 3600).toFixed(1)}h`
}

function formatChainName(name: string): string {
  return name ? name.toLowerCase().replace(/\s+/g, "_") : "unknown"
}

function getPosition(value: number, minValue: number, maxValue: number): number {
  const range = maxValue - minValue || 1
  return ((value - minValue) / range) * 100
}

function getRouteScale(item: any): { min: number; max: number } {
  // Get all values for this route to find true min/max range
  const allValues = [
    item.packetRecv.p5,
    item.packetRecv.median,
    item.packetRecv.p95,
    item.writeAck.p5,
    item.writeAck.median,
    item.writeAck.p95,
    item.packetAck.p5,
    item.packetAck.median,
    item.packetAck.p95,
  ].filter(v => v != null && !isNaN(v))

  const minValue = Math.min(...allValues)
  const maxValue = Math.max(...allValues)

  return {
    min: minValue,
    max: maxValue,
  }
}

function getSqrtPosition(value: number, minValue: number, maxValue: number): number {
  if (maxValue <= minValue) {
    return 0
  }
  const normalizedValue = Math.max(0, value - minValue)
  const normalizedMax = maxValue - minValue
  return (Math.sqrt(normalizedValue) / Math.sqrt(normalizedMax)) * 100
}

function getTimeFromSqrtPosition(position: number, minValue: number, maxValue: number): number {
  if (maxValue <= minValue || isNaN(maxValue) || isNaN(minValue)) {
    return minValue || 0
  }
  const normalizedMax = maxValue - minValue
  const normalizedPosition = Math.max(0, Math.min(1, position / 100))
  const timeValue = (normalizedPosition * normalizedPosition) * normalizedMax + minValue
  return isNaN(timeValue) ? minValue : timeValue
}

function getEventPositions(eventType: any, routeScale: { min: number; max: number }): {
  p5Pos: number
  medianPos: number
  p95Pos: number
} {
  const p5Pos = getSqrtPosition(eventType.p5, routeScale.min, routeScale.max)
  const medianPos = getSqrtPosition(eventType.median, routeScale.min, routeScale.max)
  const p95Pos = getSqrtPosition(eventType.p95, routeScale.min, routeScale.max)

  return {
    p5Pos,
    medianPos,
    p95Pos,
  }
}

let hoveredRoute = $state(-1)
let scrubberX = $state(0)
let scrubberY = $state(0)
let scrubberTime = $state(0)
let scrubberOffsetX = $state(0)

// Reset destination when source changes and current destination is no longer available
$effect(() => {
  if (
    selectedDestination && availableDestinations.length > 0
    && !availableDestinations.includes(selectedDestination)
  ) {
    selectedDestination = ""
  }
})

function handleMouseMove(event: MouseEvent, routeIndex: number): void {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top
  const percentage = Math.max(0, Math.min(100, (x / rect.width) * 100))

  // Use the same route scale that's used for visual positioning
  const item = currentData[routeIndex]
  const routeScale = getRouteScale(item)
  const time = getTimeFromSqrtPosition(percentage, routeScale.min, routeScale.max)

  hoveredRoute = routeIndex
  scrubberX = percentage
  scrubberY = y
  scrubberOffsetX = percentage
  scrubberTime = time
}

function handleMouseLeave() {
  hoveredRoute = -1
}
</script>

<div class="h-full p-0 bg-zinc-950 border border-zinc-800">
  <div class="flex flex-col h-full font-mono">
    <header class="flex items-center justify-between p-2 border-b border-zinc-800">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500 text-xs">$</span>
        <h3 class="text-xs text-zinc-300 font-semibold">latency-boxplots</h3>
        <span class="text-zinc-600 text-xs">--events=recv,write,ack</span>
        {#if selectedSource}
          <span class="text-zinc-600 text-xs">--source={formatChainName(selectedSource)}</span>
        {/if}
        {#if selectedDestination}
          <span class="text-zinc-600 text-xs">--dest={formatChainName(selectedDestination)}</span>
        {/if}
      </div>
      <div class="text-xs text-zinc-500">
        {#if !hasData}
          loading...
        {/if}
      </div>
    </header>

    <div class="pt-2 px-2">
      <div class="flex flex-col sm:flex-row gap-2 mb-2">
        <div class="flex items-center gap-1">
          <span class="text-zinc-600 text-xs font-mono">source:</span>
          <select
            bind:value={selectedSource}
            class="px-2 py-1 text-xs font-mono bg-zinc-900 border border-zinc-700 text-zinc-300 hover:border-zinc-600 focus:border-zinc-500 focus:outline-none transition-colors"
          >
            <option value="">select</option>
            {#each allSources as source}
              <option value={source}>{formatChainName(source)}</option>
            {/each}
          </select>
        </div>

        <div class="flex items-center gap-1">
          <span class="text-zinc-600 text-xs font-mono">destination:</span>
          <select
            bind:value={selectedDestination}
            class="px-2 py-1 text-xs font-mono bg-zinc-900 border border-zinc-700 text-zinc-300 hover:border-zinc-600 focus:border-zinc-500 focus:outline-none transition-colors"
          >
            <option value="">select</option>
            {#each availableDestinations as destination}
              <option value={destination}>{formatChainName(destination)}</option>
            {/each}
          </select>
        </div>

        {#if selectedSource || selectedDestination}
          <button
            class="px-2 py-1 text-xs font-mono border border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300 transition-colors"
            on:click={() => {
              selectedSource = ""
              selectedDestination = ""
            }}
            title="Clear all filters"
          >
            clear
          </button>
        {/if}
      </div>
    </div>

    <main class="flex-1 flex flex-col p-2 min-h-0">
      <div class="text-xs text-zinc-500 font-mono font-medium mb-2">
        event_latency: {currentData.length} routes
        {#if selectedSource || selectedDestination}
          <span class="text-zinc-600">
            (filtered from {
              pipe(
                latencyData,
                Option.map(data => data.length),
                Option.getOrElse(() => 0),
              )
            })
          </span>
        {/if}
      </div>

      <div class="flex-1 overflow-y-auto">
        {#if !hasData}
          <!-- Loading/No Data State - Show Skeletons -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4">
            {#each Array(4) as _, index}
              <!-- Mobile skeleton -->
              <div class="flex flex-col sm:hidden group mb-4">
                <div class="flex justify-between items-start mb-2">
                  <Skeleton class="w-32 h-3" />
                </div>

                <div class="grid grid-cols-3 gap-1 mb-2">
                  {#each Array(3) as _}
                    <div class="bg-zinc-800 border border-zinc-700 p-2 text-center">
                      <Skeleton class="w-8 h-2 mb-1 mx-auto" />
                      <Skeleton class="w-12 h-2 mb-0.5 mx-auto" />
                      <Skeleton class="w-10 h-2 mb-0.5 mx-auto" />
                      <Skeleton class="w-12 h-2 mx-auto" />
                    </div>
                  {/each}
                </div>

                <div class="bg-zinc-900 border border-zinc-800 p-2">
                  <div class="space-y-1">
                    {#each Array(3) as _}
                      <div class="flex items-center">
                        <div class="w-full relative h-6">
                          <Skeleton class="w-full h-1" />
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>

              <!-- Desktop skeleton -->
              <div class="hidden sm:block group mb-6">
                <div class="flex items-center justify-between mb-2">
                  <Skeleton class="w-40 h-3" />
                </div>

                <div class="grid grid-cols-3 gap-2 mb-2">
                  {#each Array(3) as _}
                    <div class="bg-zinc-800 border border-zinc-700 p-2 text-center">
                      <Skeleton class="w-10 h-2 mb-1 mx-auto" />
                      <Skeleton class="w-16 h-2 mb-0.5 mx-auto" />
                      <Skeleton class="w-14 h-2 mb-0.5 mx-auto" />
                      <Skeleton class="w-16 h-2 mx-auto" />
                    </div>
                  {/each}
                </div>

                <div class="bg-zinc-900 border border-zinc-800 p-2">
                  <div class="space-y-1">
                    {#each Array(3) as _}
                      <div class="flex items-center">
                        <div class="w-full relative h-6">
                          <Skeleton class="w-full h-1" />
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4">
            {#each currentData as item, index}
              <div class="flex flex-col sm:hidden group mb-4">
                <div class="flex justify-between items-start mb-2">
                  <div class="text-xs text-zinc-300 truncate flex-shrink-0">
                    <div class="font-medium">
                      {formatChainName(item.sourceName)} → {formatChainName(item.destinationName)}
                    </div>
                  </div>
                </div>

                <div class="grid grid-cols-3 gap-1 mb-2">
                  {#each item.eventTypes as eventType}
                    <div class="bg-zinc-800 border border-zinc-700 p-2 text-center">
                      <div class="text-[8px] text-zinc-500 font-mono mb-1">
                        {eventType.shortLabel}
                      </div>
                      <div class="text-[8px] text-zinc-400 font-mono">
                        P5: {formatLatency(eventType.p5)}
                      </div>
                      <div class="text-[8px] text-blue-300 font-mono font-medium">
                        Med: {formatLatency(eventType.median)}
                      </div>
                      <div class="text-[8px] text-zinc-400 font-mono">
                        P95: {formatLatency(eventType.p95)}
                      </div>
                    </div>
                  {/each}
                </div>

                <div class="space-y-1 relative bg-zinc-900 border border-zinc-800 p-2">
                  <div
                    class="relative w-full h-full"
                    on:mousemove={(e) => handleMouseMove(e, index)}
                    on:mouseleave={handleMouseLeave}
                  >
                    {#if hoveredRoute === index}
                      <div
                        class="absolute top-0 bottom-0 w-0.5 bg-zinc-400 z-20 pointer-events-none"
                        style="left: {scrubberX}%"
                      >
                      </div>
                      <div
                        class="absolute bg-zinc-800 border border-zinc-600 px-1 py-0.5 text-[8px] text-zinc-300 font-mono z-30 pointer-events-none"
                        style="left: {scrubberX}%; top: {scrubberY - 25}px; transform: translateX({scrubberX < 25 ? '0%' : scrubberX > 75 ? '-100%' : '-50%'})"
                      >
                        {formatLatency(scrubberTime)}
                      </div>
                    {/if}

                    {#each item.eventTypes as eventType}
                      {@const routeScale = getRouteScale(item)}
                      {@const positions = getEventPositions(eventType, routeScale)}
                      <div class="flex items-center">
                        <div class="w-full relative h-6">
                          <div
                            class="absolute h-0.5 bg-zinc-500"
                            style="left: {positions.p5Pos}%; 
                          width: {Math.max(0.5, positions.p95Pos - positions.p5Pos).toFixed(1)}%;
                          top: 11.5px;"
                          >
                          </div>

                          <div
                            class="absolute w-0.5 h-4 top-1 bg-blue-400 z-10"
                            style="left: {positions.medianPos}%"
                            title="{eventType.label} Median: {formatLatency(eventType.median)}"
                          >
                          </div>

                          <div
                            class="absolute w-0.5 h-3 top-1.5 bg-zinc-500"
                            style="left: {positions.p5Pos}%"
                            title="P5: {formatLatency(eventType.p5)}"
                          >
                          </div>

                          <div
                            class="absolute w-0.5 h-3 top-1.5 bg-zinc-500"
                            style="left: {positions.p95Pos}%"
                            title="P95: {formatLatency(eventType.p95)}"
                          >
                          </div>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>

              <div class="hidden sm:block group mb-6">
                <div class="flex items-center justify-between mb-2">
                  <div class="text-xs text-zinc-300 font-medium">
                    {formatChainName(item.sourceName)} → {formatChainName(item.destinationName)}
                  </div>
                </div>

                <div class="grid grid-cols-3 gap-2 mb-2">
                  {#each item.eventTypes as eventType}
                    <div class="bg-zinc-800 border border-zinc-700 p-2 text-center">
                      <div class="text-xs text-zinc-500 font-mono mb-1">{eventType.shortLabel}</div>
                      <div class="text-[10px] text-zinc-400 font-mono">
                        P5: {formatLatency(eventType.p5)}
                      </div>
                      <div class="text-[10px] text-blue-300 font-mono font-medium">
                        Med: {formatLatency(eventType.median)}
                      </div>
                      <div class="text-[10px] text-zinc-400 font-mono">
                        P95: {formatLatency(eventType.p95)}
                      </div>
                    </div>
                  {/each}
                </div>

                <div class="space-y-1 relative bg-zinc-900 border border-zinc-800 p-2">
                  <div
                    class="relative w-full h-full"
                    on:mousemove={(e) => handleMouseMove(e, index)}
                    on:mouseleave={handleMouseLeave}
                  >
                    {#if hoveredRoute === index}
                      <div
                        class="absolute top-0 bottom-0 w-0.5 bg-zinc-400 z-20 pointer-events-none"
                        style="left: {scrubberX}%"
                      >
                      </div>
                      <div
                        class="absolute bg-zinc-800 border border-zinc-600 px-1 py-0.5 text-xs text-zinc-300 font-mono z-30 pointer-events-none"
                        style="left: {scrubberX}%; top: {scrubberY - 25}px; transform: translateX({scrubberX < 25 ? '0%' : scrubberX > 75 ? '-100%' : '-50%'})"
                      >
                        {formatLatency(scrubberTime)}
                      </div>
                    {/if}

                    {#each item.eventTypes as eventType}
                      {@const routeScale = getRouteScale(item)}
                      {@const positions = getEventPositions(eventType, routeScale)}
                      <div class="flex items-center">
                        <div class="w-full relative h-6">
                          <div
                            class="absolute h-0.5 bg-zinc-500"
                            style="left: {positions.p5Pos}%; 
                          width: {Math.max(0.5, positions.p95Pos - positions.p5Pos).toFixed(1)}%;
                          top: 11.5px;"
                          >
                          </div>

                          <div
                            class="absolute w-0.5 h-4 top-1 bg-blue-400 z-10"
                            style="left: {positions.medianPos}%"
                            title="{eventType.label} Median: {formatLatency(eventType.median)}"
                          >
                          </div>

                          <div
                            class="absolute w-0.5 h-3 top-1.5 bg-zinc-500"
                            style="left: {positions.p5Pos}%"
                            title="P5: {formatLatency(eventType.p5)}"
                          >
                          </div>

                          <div
                            class="absolute w-0.5 h-3 top-1.5 bg-zinc-500"
                            style="left: {positions.p95Pos}%"
                            title="P95: {formatLatency(eventType.p95)}"
                          >
                          </div>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </main>
  </div>
</div>

<style>
/* Custom scrollbar styling */
.overflow-y-auto::-webkit-scrollbar {
width: 4px;
}

.overflow-y-auto::-webkit-scrollbar-track {
background: #27272a;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
background: #52525b;
border-radius: 2px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
background: #71717a;
}
</style>
