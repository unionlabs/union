<script lang="ts">
import NetworkVisualizer from "$lib/components/NetworkVisualizer.svelte"
import TerminalLog from "$lib/components/TerminalLog.svelte"
import TransferStats from "$lib/components/TransferStats.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { onDestroy, onMount } from "svelte"
import { createTransferPollingMachine } from "./machine"

let machine: any = null
let transfers: TransferListItem[] = []
let selectedFromChain: string | null = null
let selectedToChain: string | null = null

function handleChainSelection(fromChain: string | null, toChain: string | null) {
  selectedFromChain = fromChain
  selectedToChain = toChain
}

onMount(() => {
  machine = createTransferPollingMachine(100) // Increased limit for better filtering

  machine.onNewTransfers((newTransfers: TransferListItem[]) => {
    // Add new transfers to the array for components to consume
    transfers = [...transfers, ...newTransfers]
  })
})

onDestroy(() => {
  if (machine) {
    machine.destroy()
  }
})
</script>

<div class="grid grid-cols-1 lg:grid-cols-3 gap-4 h-full p-4 bg-zinc-950">
  <!-- Network Visualizer - first on mobile, right side on desktop (spans 2 columns) -->
  <div class="order-1 lg:order-3 lg:col-span-2 min-h-0">
    <NetworkVisualizer
      {transfers}
      onChainSelection={handleChainSelection}
    />
  </div>

  <!-- Stats - second on mobile, spans full width on desktop -->
  <div class="order-2 lg:order-1 lg:col-span-3">
    <TransferStats {transfers} />
  </div>

  <!-- Terminal Log - third on mobile, left side on desktop -->
  <div class="order-3 lg:order-2 lg:col-span-1 min-h-0">
    <TerminalLog
      {transfers}
      {selectedFromChain}
      {selectedToChain}
    />
  </div>
</div>
