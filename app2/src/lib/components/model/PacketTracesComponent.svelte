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

  const isLeft = nextColumn < currentColumn
  const start = currentColumn * 2 + (isLeft ? 2 : 3)
  const end = nextColumn * 2 + (isLeft ? 2 : 3)

  return {
    gridColumn: `${Math.min(start, end)} / ${Math.max(start, end)}`,
    isLeft
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
  <div class="mt-12">
      <div class="grid mt-4 size-[120%] mx-[-10%]" style="grid-template-columns: {getGridTemplateColumns(positions.columns)}">


        <!-- Background grid with lines -->
        {#each Array(positions.columns) as _, i}
          <div class="bg-zinc-800" style="grid-row: 2 / 100; grid-column: {i * 2 + 2}" ></div>
        {/each}
        
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
          <div class="text-center {Option.isSome(centerChain) ? 'col-start-5' : 'col-start-3'} col-span-3 row-1">
            <ChainComponent chain={rightChain.value} />
          </div>
        {/if}
        <!-- Traces and arrows -->

        {#each packetTraces as trace, i}
          {@const chain = getChain(chainsList, trace.chain.universal_chain_id)}
          {@const column = getTraceColumn(trace, positions)}
          {@const nextTrace = packetTraces[i + 1]}
          {@const prevTrace = Option.fromNullable(i > 0 ? packetTraces[i - 1] : null)}
          {@const arrowSpan = getArrowSpan(trace, nextTrace, positions)}
          
            <!-- Trace card -->

            <div class="pb-1 rounded-lg col-span-3 flex flex-col items-center" 
              style="grid-row: {i * 2 + 2}; grid-column-start: {column * 2 + 1}  ">
            {#if Option.isSome(trace.height) && !(Option.isSome(prevTrace) && Option.isSome(prevTrace.value.height) && prevTrace.value.height.value === trace.height.value)}
              <div class="text-zinc-500 pt-2 flex text-xs flex-col items-center">
              <div>{trace.height.value}</div>
              {#if Option.isSome(trace.timestamp)}
                <DateTimeComponent class="text-zinc-500" showDate={false} value={trace.timestamp.value} />
              {/if}
              </div>

            {/if}
              <div class="bg-zinc-800 flex flex-col items-center rounded px-2 py-1">
                <div class="font-semibold text-sm">
                  {toTraceName(trace.type)}
                </div>

                {#if Option.isSome(trace.height) && Option.isSome(trace.timestamp) && Option.isSome(trace.transaction_hash) && Option.isSome(chain)}
                  <div class="text-xs text-zinc-400">
                    <TransactionHashComponent hash={trace.transaction_hash.value} />
                  </div>
                {/if}
              </div>
            </div>

            {#if arrowSpan}
              <div class="flex py-2 items-center {arrowSpan.isLeft ? 'flex-row-reverse' : 'flex-row'}" style="grid-row: {i * 2 + 3}; grid-column: {arrowSpan.gridColumn};">
                  <div class="flex-1 h-0.5 bg-zinc-700">
                  </div>
                    <div class="border-[5px] border-transparent {arrowSpan.isLeft ? 'dark:border-r-zinc-700' : 'border-l-zinc-400 dark:border-l-zinc-700'}" ></div>
              </div>
            {/if}
        {/each}
        <!-- spacer !-->
        <div class="h-8" style="grid-row: 40;"></div>
       
      </div>

  </div>
{/if}
