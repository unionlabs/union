<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { getChain, PacketTrace } from "@unionlabs/sdk/schema"
import { chains } from "$lib/stores/chains.svelte"
import { cn } from "$lib/utils"
import Label from "../ui/Label.svelte"
import { Option } from "effect"
import DateTimeComponent from "../ui/DateTimeComponent.svelte"
import TransactionHashComponent from "./TransactionHashComponent.svelte"
import BlockHashComponent from "./BlockHashComponent.svelte"
import ChainComponent from "./ChainComponent.svelte"
import { PACKET_TRACE_DISPLAY_NAMES } from "$lib/constants/packet-trace-names"

type Props = HTMLAttributes<HTMLDivElement> & {
  packetTraces: ReadonlyArray<PacketTrace>
}

const { packetTraces }: Props = $props()

const toTraceName = (type: string) =>
  type in PACKET_TRACE_DISPLAY_NAMES ? PACKET_TRACE_DISPLAY_NAMES[type] : type

function getChainPositions(traces: ReadonlyArray<PacketTrace>) {
  const hasL2Update = traces.some(t => t.type === "PACKET_SEND_LC_UPDATE_L2")
  const sendChain = traces.find(t => t.type === "PACKET_SEND")?.chain
  const recvChain = traces.find(t => t.type === "PACKET_RECV")?.chain
  const l2Chain = hasL2Update
    ? traces.find(t => t.type === "PACKET_SEND_LC_UPDATE_L2")?.chain
    : undefined

  return {
    left: sendChain,
    center: l2Chain,
    right: recvChain,
    columns: hasL2Update ? 3 : 2
  }
}

function getTraceColumn(trace: PacketTrace, positions: ReturnType<typeof getChainPositions>) {
  if (trace.chain.universal_chain_id === positions.left?.universal_chain_id) return 0
  if (positions.center && trace.chain.universal_chain_id === positions.center.universal_chain_id)
    return 1
  if (trace.chain.universal_chain_id === positions.right?.universal_chain_id)
    return positions.columns - 1
  return 1
}

function getGridTemplateColumns(columns: number) {
  const parts = []
  for (let i = 0; i < columns; i++) {
    parts.push("1fr", "2px")
    if (i === columns - 1) {
      parts.push("1fr")
    }
  }
  return parts.join(" ")
}

function getArrowSpan(
  currentTrace: PacketTrace,
  nextTrace: PacketTrace | undefined,
  positions: ReturnType<typeof getChainPositions>
) {
  if (!nextTrace) return null

  const currentColumn = getTraceColumn(currentTrace, positions)
  const nextColumn = getTraceColumn(nextTrace, positions)

  // Don't draw arrow if on same column
  if (currentColumn === nextColumn) return null

  const start = currentColumn * 2 + 2
  const end = nextColumn * 2 + 2

  return {
    gridColumn: `${Math.min(start, end)} / ${Math.max(start, end)}`,
    isLeft: nextColumn < currentColumn
  }
}
</script>


{#if packetTraces.length > 0 && Option.isSome(chains.data)}
{@const chainsList = chains.data.value}
{@const positions = getChainPositions(packetTraces)}
{@const leftChain = Option.fromNullable(positions.left?.universal_chain_id)
  .pipe(Option.flatMap((id) => getChain(chainsList, id)))}
{@const centerChain = Option.fromNullable(positions.center?.universal_chain_id)
  .pipe(Option.flatMap((id) => getChain(chainsList, id)))}
{@const rightChain = Option.fromNullable(positions.right?.universal_chain_id)
  .pipe(Option.flatMap((id) => getChain(chainsList, id)))}
  <div>
    <Label>Packet Trace</Label>
    
      <!-- Chain headers -->
      <div class=" grid mb-4 size-full" style="grid-template-columns: {getGridTemplateColumns(positions.columns)}">
        <!-- Chain names with lines -->
        {#if Option.isSome(leftChain)}
          <div class="text-center col-start-1 col-span-3 row-1">
            <ChainComponent chain={leftChain.value} />
          </div>
        {/if}
        
        {#if Option.isSome(centerChain)}
          <div class="text-center col-start-3 col-span-3 row-1">
            <ChainComponent chain={centerChain.value} />
          </div>
        {/if}
        
        {#if Option.isSome(rightChain)}
          <div class="text-center col-start-5 col-span-3 row-1">
            <ChainComponent chain={rightChain.value} />
          </div>
        {/if}

        <!-- Background grid with lines -->
        {#each Array(positions.columns) as _, i}
          <div class="bg-zinc-800 row-start-1 row-span-10" style="grid-column: {i * 2 + 2}" ></div>
        {/each}

        <!-- Traces and arrows -->

        {#each packetTraces as trace, i}
          {@const chain = getChain(chainsList, trace.chain.universal_chain_id)}
          {@const column = getTraceColumn(trace, positions)}
          {@const nextTrace = packetTraces[i + 1]}
          {@const arrowSpan = getArrowSpan(trace, nextTrace, positions)}
          
            <!-- Trace card -->
            <div class=" p-2 rounded-lg col-span-3 flex justify-center" 
              style="grid-column-start: {column * 2 + 1}  ">
              <div class="bg-zinc-800 rounded px-4 py-2">
                <div class="flex items-center justify-between">
                  <span class="font-bold text-zinc-900 dark:text-zinc-100">
                    {toTraceName(trace.type)}
                  </span>
                </div>

                {#if Option.isSome(trace.height) && Option.isSome(trace.timestamp) && Option.isSome(trace.transaction_hash) && Option.isSome(chain)}
                  <div class="text-sm text-zinc-600 dark:text-zinc-400">
                    <p><DateTimeComponent value={trace.timestamp.value} /> <span>in block {trace.height.value}</span></p>
                    <TransactionHashComponent hash={trace.transaction_hash.value} />
                  </div>
                {/if}
              </div>
            </div>

            {#if arrowSpan}
              <div class="MARKER" style="grid-column: {arrowSpan.gridColumn};">
                  {JSON.stringify(arrowSpan)}
                  <div class="w-full h-0.5 bg-zinc-400 dark:bg-zinc-600">
                    <!--<div class="absolute {arrowSpan.isLeft ? 'left-0' : 'right-0'} top-[-4px] border-[5px] border-transparent {arrowSpan.isLeft ? 'border-r-zinc-400 dark:border-r-zinc-600' : 'border-l-zinc-400 dark:border-l-zinc-600'}" ></div>-->
                  </div>
              </div>
            {/if}
        {/each}
        
      </div>

      <!--
      <div class="absolute inset-0 grid overflow-hidden" style="grid-template-columns: {getGridTemplateColumns(positions.columns)}">
      </div>
      !-->

      <!--
      <div class="relative grid gap-4 p-4" style="grid-template-columns: {getGridTemplateColumns(positions.columns)}">
        {#each packetTraces as trace, i}
          {@const chain = getChain(chainsList, trace.chain.universal_chain_id)}
          {@const column = getTraceColumn(trace, positions)}
          {@const nextTrace = packetTraces[i + 1]}
          {@const arrowSpan = getArrowSpan(trace, nextTrace, positions)}
          
          <div class="contents">
            <div class=" p-2 rounded-lg bg-zinc-100 dark:bg-zinc-800" 
              style="grid-row: {i + 1}; grid-column: {column * 2 - 1}">
              <div class="flex items-center justify-between">
                <span class="font-bold text-zinc-900 dark:text-zinc-100">
                  {toTraceName(trace.type)}
                </span>
              </div>

              {#if Option.isSome(trace.height) && Option.isSome(trace.timestamp) && Option.isSome(trace.transaction_hash) && Option.isSome(chain)}
                <div class="text-sm text-zinc-600 dark:text-zinc-400">
                  <p><DateTimeComponent value={trace.timestamp.value} /> <span>in block {trace.height.value}</span></p>
                  <TransactionHashComponent hash={trace.transaction_hash.value} />
                </div>
              {/if}
            </div>

            {#if arrowSpan}
              <div style="grid-row: {i + 1}; {arrowSpan.gridColumn}">
                <div class="absolute inset-0 flex items-center">
                  <div class="w-full h-0.5 bg-zinc-400 dark:bg-zinc-600">
                    <div class="absolute {arrowSpan.isLeft ? 'left-0' : 'right-0'} top-[-4px] border-[5px] border-transparent {arrowSpan.isLeft ? 'border-r-zinc-400 dark:border-r-zinc-600' : 'border-l-zinc-400 dark:border-l-zinc-600'}" />
                  </div>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
      !-->
  </div>
{/if}
