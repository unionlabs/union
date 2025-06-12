<script lang="ts">
import { chains } from "$lib/stores/chains.svelte"
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import { onMount } from "svelte"
import { transactionAudio } from "../../routes/test/audio"
import Card from "./ui/Card.svelte"

let {
  transfers = [],
  selectedFromChain = null,
  selectedToChain = null,
}: {
  transfers: TransferListItem[]
  selectedFromChain?: string | null
  selectedToChain?: string | null
} = $props()

interface LogEntry {
  id: number
  timestamp: string
  type: string
  message: string
  sourceChain: string
  destChain: string
  hash: string
  sender?: string | undefined
  receiver?: string | undefined
  sourceChainId?: string | undefined
  destChainId?: string | undefined
}

let allLogs: LogEntry[] = $state([])
let logContainer: HTMLElement
let processedCount = $state(0)
let logIdCounter = $state(0)

// Optimization: Cache chain display names to avoid repeated lookups
const chainDisplayNameCache = new Map<string, string>()

const getChainDisplayName = (universalChainId: string): string => {
  // Check cache first
  if (chainDisplayNameCache.has(universalChainId)) {
    return chainDisplayNameCache.get(universalChainId)!
  }

  let displayName = universalChainId
  
  if (Option.isSome(chains.data)) {
    const chain = chains.data.value.find(c => c.universal_chain_id === universalChainId)
    if (chain) {
      displayName = chain.display_name || chain.chain_id
    }
  } else {
    // Fallback to simple mapping if chains store not loaded
    const simpleNames: Record<string, string> = {
      "union-testnet-8": "Union",
      "osmo-test-5": "Osmosis",
      "sepolia-1": "Ethereum",
      "stride-internal-1": "Stride",
    }
    displayName = simpleNames[universalChainId] || universalChainId
  }

  // Cache the result
  chainDisplayNameCache.set(universalChainId, displayName)
  return displayName
}

const formatHash = (hash: string): string => {
  return hash ? `${hash.slice(0, 8)}...${hash.slice(-4)}` : "N/A"
}

// Optimization: Pre-filter logs based on chain selection to reduce reactive computation
let filteredLogs = $derived.by(() => {
  if (!selectedFromChain && !selectedToChain) {
    return allLogs.slice(0, 100) // Only show recent 100 logs for performance
  }

  const filtered = allLogs.filter(log => {
    const sourceId = log.sourceChainId
    const destId = log.destChainId

    // If both chains are selected, must match the exact route
    if (selectedFromChain && selectedToChain) {
      return sourceId === selectedFromChain && destId === selectedToChain
    }

    // If only one chain is selected, must involve that chain
    if (selectedFromChain) {
      return sourceId === selectedFromChain || destId === selectedFromChain
    }

    if (selectedToChain) {
      return sourceId === selectedToChain || destId === selectedToChain
    }

    return true
  })

  return filtered.slice(0, 100) // Limit to 100 filtered results
})

// Optimization: Batch log additions to reduce reactive updates
let pendingLogs: LogEntry[] = []
let batchTimeoutId: ReturnType<typeof setTimeout> | null = null

const flushPendingLogs = () => {
  if (pendingLogs.length === 0) return
  
  allLogs = [...pendingLogs, ...allLogs].slice(0, 500) // Keep max 500 logs
  pendingLogs = []
  batchTimeoutId = null
}

const addLog = (
  type: string, 
  sourceChain: string, 
  destChain: string, 
  hash: string, 
  sender?: string, 
  receiver?: string,
  sourceChainId?: string,
  destChainId?: string
) => {
  const timestamp = new Date().toLocaleTimeString()
  logIdCounter++

  const logEntry: LogEntry = {
    id: logIdCounter,
    timestamp,
    type,
    message: `${sourceChain} → ${destChain}`,
    sourceChain,
    destChain,
    hash,
    sender,
    receiver,
    sourceChainId,
    destChainId,
  }

  pendingLogs.push(logEntry)

  // Batch updates - flush after 16ms (next frame) or when we have 10+ pending
  if (batchTimeoutId === null) {
    if (pendingLogs.length >= 10) {
      flushPendingLogs()
    } else {
      batchTimeoutId = setTimeout(flushPendingLogs, 16)
    }
  }
}

// Check if transfer matches selected chain filter (for sound filtering)
const shouldPlaySound = (transfer: any): boolean => {
  if (!selectedFromChain && !selectedToChain) {
    return true
  }

  const sourceId = transfer.source_chain?.universal_chain_id
  const destId = transfer.destination_chain?.universal_chain_id

  // If both chains are selected, must match the exact route
  if (selectedFromChain && selectedToChain) {
    return sourceId === selectedFromChain && destId === selectedToChain
  }

  // If only one chain is selected, must involve that chain
  if (selectedFromChain) {
    return sourceId === selectedFromChain || destId === selectedFromChain
  }

  if (selectedToChain) {
    return sourceId === selectedToChain || destId === selectedToChain
  }

  return true
}

// Process new transfers reactively
$effect(() => {
  if (transfers.length > processedCount) {
    const newTransfers = transfers.slice(processedCount)
    newTransfers.forEach((transfer: any) => {
      // Only play sound for transfers that match the current filter
      if (shouldPlaySound(transfer)) {
        const value = parseFloat(transfer.base_amount?.toString() || "0") || 1
        const sourceChainId = transfer.source_chain?.universal_chain_id
        const destChainId = transfer.destination_chain?.universal_chain_id
        transactionAudio.playSound(value, sourceChainId, destChainId)
      }

      // Pre-compute display values to avoid repeated work
      const sourceChain = getChainDisplayName(
        transfer.source_chain?.universal_chain_id || "unknown",
      )
      const destChain = getChainDisplayName(
        transfer.destination_chain?.universal_chain_id || "unknown",
      )
      const hash = formatHash(transfer.packet_hash)
      const sender = transfer.sender_canonical
        ? `${transfer.sender_canonical.slice(0, 8)}...${transfer.sender_canonical.slice(-4)}`
        : undefined
      const receiver = transfer.receiver_canonical
        ? `${transfer.receiver_canonical.slice(0, 8)}...${transfer.receiver_canonical.slice(-4)}`
        : undefined

      addLog(
        "transfer",
        sourceChain,
        destChain,
        hash,
        sender,
        receiver,
        transfer.source_chain?.universal_chain_id,
        transfer.destination_chain?.universal_chain_id,
      )
    })
    processedCount = transfers.length
  }
})

onMount(() => {
  addLog("system", "Terminal", "Ready", "waiting for transfers...")
})

// Clear logs and cache
const clearLogs = () => {
  allLogs = []
  pendingLogs = []
  chainDisplayNameCache.clear()
  if (batchTimeoutId !== null) {
    clearTimeout(batchTimeoutId)
    batchTimeoutId = null
  }
}
</script>

<Card class="h-full flex flex-col">
  <div class="flex flex-col h-full text-zinc-300 font-mono text-sm overflow-hidden">
    {#if selectedFromChain || selectedToChain}
      <div class="px-2 py-1 bg-zinc-800 border-b border-zinc-700 text-xs text-amber-400">
        {#if selectedFromChain && selectedToChain}
          Filtering: {getChainDisplayName(selectedFromChain)} → {
            getChainDisplayName(selectedToChain)
          }
        {:else if selectedFromChain}
          Filtering: {getChainDisplayName(selectedFromChain)} (any direction)
        {:else if selectedToChain}
          Filtering: {getChainDisplayName(selectedToChain)} (any direction)
        {/if}
      </div>
    {/if}
    <div
      class="flex-1 overflow-y-auto leading-relaxed scrollbar-thin scrollbar-track-zinc-900 scrollbar-thumb-zinc-600 hover:scrollbar-thumb-zinc-500 min-h-0 p-2"
      bind:this={logContainer}
    >
      {#each filteredLogs as log (log.id)}
        <div class="text-xs mb-2">
          <div class="text-zinc-300 font-mono">{log.message}</div>
          <div class="text-zinc-300 font-mono">tx: {log.hash}</div>
          {#if log.sender}
            <div class="text-zinc-400">from: {log.sender}</div>
          {/if}
          {#if log.receiver}
            <div class="text-zinc-400">to: {log.receiver}</div>
          {/if}
        </div>
      {/each}

      {#if filteredLogs.length === 0}
        <div class="flex items-center justify-center h-full text-zinc-500 flex-col gap-2">
          <div>Waiting for transfers...</div>
          <div class="animate-pulse font-bold">_</div>
        </div>
      {/if}
    </div>
  </div>
</Card>
