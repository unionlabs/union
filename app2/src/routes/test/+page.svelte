<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import NetworkVisualizer from "$lib/components/NetworkVisualizer.svelte"
  import TerminalLog from "$lib/components/TerminalLog.svelte"
  import TransferStats from "$lib/components/TransferStats.svelte"
  import { createTransferPollingMachine } from "./machine"
  import type { TransferListItem } from '@unionlabs/sdk/schema'
  import Card from '$lib/components/ui/Card.svelte';
  import Sections from '$lib/components/ui/Sections.svelte';
  
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


  <Card class="h-full rounded-none border-none bg-zinc-950">
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 h-full p-4">
      <!-- Stats - spans full width on mobile, 1 column on desktop -->
      <div class="lg:col-span-3">
        <TransferStats {transfers} />
      </div>
      
      <!-- Terminal Log - left side on desktop -->
      <div class="lg:col-span-1 min-h-0">
        <TerminalLog {transfers} {selectedFromChain} {selectedToChain} />
      </div>
      
      <!-- Network Visualizer - right side on desktop, spans 2 columns -->
      <div class="lg:col-span-2 min-h-0">
        <NetworkVisualizer {transfers} onChainSelection={handleChainSelection} />
      </div>
    </div>
  </Card>

 