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
  if (trace.chain.universal_chain_id === positions.left?.universal_chain_id) return 1
  if (positions.center && trace.chain.universal_chain_id === positions.center.universal_chain_id)
    return 2
  if (trace.chain.universal_chain_id === positions.right?.universal_chain_id)
    return positions.columns
  return 1
}
</script>


{#if packetTraces.length > 0 && Option.isSome(chains.data)}
{@const chainsList = chains.data.value}
{@const positions = getChainPositions(packetTraces)}
{@const leftChain = Option.fromNullable(positions.left?.universal_chain_id)
  .pipe(Option.map((id) => getChain(chainsList, id)))}
{@const centerChain = Option.fromNullable(positions.center?.universal_chain_id)
  .pipe(Option.map((id) => getChain(chainsList, id)))}
{@const rightChain = Option.fromNullable(positions.right?.universal_chain_id)
  .pipe(Option.map((id) => getChain(chainsList, id)))}
  <div>
    <Label>Packet Trace</Label>
    
    <div class="grid gap-4 p-4 mt-4" style="grid-template-columns: repeat({positions.columns}, 1fr)">
      <!-- Chain headers -->
      {#if Option.isSome(leftChain) && Option.isSome(leftChain.value)}
        <div class="text-center mb-4">
          <ChainComponent chain={leftChain.value.value} />
        </div>
      {/if}
      
      {#if Option.isSome(centerChain) && Option.isSome(centerChain.value)}
        <div class="text-center mb-4">
          <ChainComponent chain={centerChain.value.value} />
        </div>
      {/if}
      
      {#if Option.isSome(rightChain) && Option.isSome(rightChain.value)}
        <div class="text-center mb-4">
          <ChainComponent chain={rightChain.value.value} />
        </div>
      {/if}

      <!-- Chain vertical lines -->
      {#each Array(positions.columns) as _, i}
        <div class="w-0.5 h-full mx-auto bg-zinc-200 dark:bg-zinc-700" />
      {/each}

      <!-- Traces -->
      {#each packetTraces as trace, i}
        {@const chain = getChain(chainsList, trace.chain.universal_chain_id)}
        {@const column = getTraceColumn(trace, positions)}
        
        <div class="relative p-2 rounded-lg bg-zinc-100 dark:bg-zinc-800" style="grid-row: {i + 2}; grid-column: {column}">
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

          <!-- Arrows -->
          {#if trace.type === "PACKET_SEND" || trace.type === "PACKET_SEND_LC_UPDATE_L2"}
            <div class="absolute h-0.5 bg-zinc-400 dark:bg-zinc-600 top-1/2 -translate-y-1/2 left-full w-[calc(100%-2rem)] after:absolute after:right-[-6px] after:top-[-4px] after:border-[5px] after:border-transparent after:border-l-zinc-400 dark:after:border-l-zinc-600" />
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}
