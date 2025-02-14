<script lang="ts">
import { cn } from "$lib/utilities/shadcn"
import Button from "$lib/components/ui/button/button.svelte"
import { type Readable, derived } from "svelte/store"
import SpinnerSvg from "./spinner-svg.svelte"
import type { Step, RawTrace, Trace, StepStatus } from "$lib/stepper-types.ts"
import { toIsoString } from "$lib/utilities/date"
import Truncate from "$lib/components/truncate.svelte"
import { createEventDispatcher } from "svelte"
import { toDisplayName } from "$lib/utilities/chains.ts"
import type { Chain } from "$lib/types"

export let traces: Array<RawTrace>
export let chains: Array<Chain>

const DISPLAY_NAMES: Record<StepStatus, Record<string, string>> = {
  COMPLETED: {
    PACKET_SEND: "Packet Sent",
    PACKET_RECV: "Packet Received",
    WRITE_ACK: "Acknowledgement Written",
    PACKET_ACK: "Packet Acknowledged",
    PACKET_SEND_LC_UPDATE_L0: "L0 Light Client Updated",
    PACKET_SEND_LC_UPDATE_L1: "L1 Light Client Updated",
    PACKET_SEND_LC_UPDATE_L2: "L2 Light Client Updated",
    WRITE_ACK_LC_UPDATE_L0: "L0 Light Client Updated (Ack)",
    WRITE_ACK_LC_UPDATE_L1: "L1 Light Client Updated (Ack)",
    WRITE_ACK_LC_UPDATE_L2: "L2 Light Client Updated (Ack)"
  },
  PENDING: {
    PACKET_SEND: "Send Packet",
    PACKET_RECV: "Receive Packet",
    WRITE_ACK: "Write Acknowledgement",
    PACKET_ACK: "Acknowledge Packet",
    PACKET_SEND_LC_UPDATE_L0: "Update L0 Light Client",
    PACKET_SEND_LC_UPDATE_L1: "Update L1 Light Client",
    PACKET_SEND_LC_UPDATE_L2: "Update L2 Light Client",
    WRITE_ACK_LC_UPDATE_L0: "Update L0 Light Client (Ack)",
    WRITE_ACK_LC_UPDATE_L1: "Update L1 Light Client (Ack)",
    WRITE_ACK_LC_UPDATE_L2: "Update L2 Light Client (Ack)"
  },
  IN_PROGRESS: {
    PACKET_SEND: "Sending Packet",
    PACKET_RECV: "Receiving Packet",
    WRITE_ACK: "Writing Acknowledgement",
    PACKET_ACK: "Acknowledging Packet",
    PACKET_SEND_LC_UPDATE_L0: "Updating L0 Light Client",
    PACKET_SEND_LC_UPDATE_L1: "Updating L1 Light Client",
    PACKET_SEND_LC_UPDATE_L2: "Updating L2 Light Client",
    WRITE_ACK_LC_UPDATE_L0: "Updating L0 Light Client (Ack)",
    WRITE_ACK_LC_UPDATE_L1: "Updating L1 Light Client (Ack)",
    WRITE_ACK_LC_UPDATE_L2: "Updating L2 Light Client (Ack)"
  },
  WARNING: {},
  ERROR: {}
}

$: pTraces = ((): Array<Trace> => {
  let processedTraces = traces.map(t => {
    let chain = chains.find(c => c.chain_id === t.chain?.chain_id)
    let explorer = chain?.explorers?.at(0)

    // For cosmos explorer, transaction hashes must not have 0x and must be all-uppercase.
    let transaction_hash =
      chain?.rpc_type === "cosmos" ? t.transaction_hash?.slice(2).toUpperCase() : t.transaction_hash

    return {
      ...t,
      transaction_hash,
      status: t.transaction_hash ? "COMPLETED" : ("PENDING" as StepStatus),
      block_url: explorer ? `${explorer.block_url}${t.height}` : null,
      transaction_url: explorer ? `${explorer.tx_url}${transaction_hash}` : null
    }
  })

  for (const [index, step] of processedTraces.entries()) {
    if (step.status === "COMPLETED") {
      const next = processedTraces.at(index + 1)
      if (!next || next.status === "COMPLETED") {
        continue
      }
      next.status = "IN_PROGRESS"
    }
  }
  return processedTraces.map(t => ({ ...t, type: DISPLAY_NAMES[t.status][t.type] ?? t.type }))
})()
</script>

<ol class="max-w-full w-full -my-4"> <!-- offset padding surplus !-->
{#each pTraces as trace, index}
  <li class="flex gap-4 w-full">
    <div class="flex flex-col items-center">
      <!-- top trace connector !-->
      <div class={cn(
          "w-1 flex-1",
          index !== 0 ?  "dark:bg-neutral-500 bg-black" : "",
          )}></div>
      <div class={cn(
        "size-12 border-4 dark:border-neutral-500 relative transition-all duration-300",
        trace.status === "PENDING" ? "bg-white dark:bg-neutral-700" :
        trace.status === "IN_PROGRESS" ? "bg-white dark:bg-neutral-700" :
        trace.status === "COMPLETED" ? "bg-accent" :
        trace.status === "ERROR" ? "bg-black" :
        trace.status === "WARNING" ? "bg-yellow-300" : ""
      )}>
        <div class={cn("absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2  rounded-full bg-black transition-all duration-300",
          trace.status === "COMPLETED" ? "w-1 h-7 rotate-45 translate-x-[2px]" :
          trace.status === "ERROR" ? "w-1 h-8 rotate-45 bg-white" :
          trace.status === "WARNING" ? "w-1 h-4 -translate-y-[12px]" : "w-2 h-2"
          )}></div>
        <div class={cn("absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-black transition-all duration-300",
          trace.status === "COMPLETED" ? "w-1 h-4 -rotate-45 -translate-x-3 -translate-y-[2px]" :
          trace.status === "ERROR" ? "w-1 h-8 -rotate-45 bg-white" :
          trace.status === "WARNING" ? "w-1 h-1 translate-y-[8px]" : "w-2 h-2"
          )}></div>
        {#if trace.status === "IN_PROGRESS"}
          <SpinnerSvg className="absolute text-accent w-8 h-8 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"/>
        {/if}
      </div>
      <!-- bottom trace connector !-->
      <div class={cn("w-1 flex-1",
      index === pTraces.length - 1 ? "bg-transparent" : "dark:bg-neutral-500 bg-black",
      index !== pTraces.length - 1  &&
      trace.status !== "ERROR" &&
      trace.status !== "WARNING" ?  "bg-black" : "")
      }></div>
    </div>
    <div class="font-bold py-4 flex flex-col min-h-[80px] max-w-[calc(100%-80px)] break-words justify-center">
      {#if trace.timestamp}
        <p class="text-xs -mb-1 text-muted-foreground">{toIsoString(new Date(trace.timestamp)).split('T')[1]} on {toDisplayName(trace.chain?.chain_id, chains)} at {#if trace.block_url}<a class="underline" target="_blank" href={trace.block_url}>{trace.height}</a>{:else}{trace.height}{/if}</p>
      {/if}
      <div>{trace.type}</div>
      {#if trace.transaction_hash}
        {#if trace.transaction_url}
          <a href={trace.transaction_url} target="_blank"  class="-mt-1 block underline text-xs text-muted-foreground"><Truncate class="underline" value={trace.transaction_hash} type="hash"/></a>
        {:else}
          <p class="text-xs text-muted-foreground"><Truncate value={trace.transaction_hash} type="hash"/></p>
        {/if}
      {/if}
    </div>
  </li>
{/each}
</ol>


