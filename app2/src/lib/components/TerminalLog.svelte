<script lang="ts">
  import { onMount } from 'svelte'
  import type { TransferListItem } from '@unionlabs/sdk/schema'
  import { chains } from '$lib/stores/chains.svelte'
  import { Option } from 'effect'
  import Card from './ui/Card.svelte'
  import { transactionAudio } from '../../routes/test/audio';
  
  let { 
    transfers = [], 
    selectedFromChain = null, 
    selectedToChain = null 
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
    data?: any
  }
  
  let allLogs: LogEntry[] = $state([])
  let logContainer: HTMLElement
  let processedCount = $state(0)
  let startTime = $state(Date.now())
  let logIdCounter = $state(0)
  
  // Filtered logs based on chain selection
  let filteredLogs = $derived.by(() => {
    if (!selectedFromChain && !selectedToChain) {
      return allLogs
    }
    
    return allLogs.filter(log => {
      if (log.type !== 'transfer' || !log.data) return true // Show system messages
      
      const sourceId = log.data.sourceChainId
      const destId = log.data.destChainId
      
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
  })

  
  const addLog = (type: string, message: string, data: any = null) => {
    const timestamp = new Date().toLocaleTimeString()
    logIdCounter++
    
    allLogs = [{ 
      id: logIdCounter,
      timestamp, 
      type, 
      message, 
      data 
    }, ...allLogs].slice(0, 500) // Keep many more logs to handle filtering
  }
  
  const getChainDisplayName = (universalChainId: string): string => {
    if (Option.isSome(chains.data)) {
      const chain = chains.data.value.find(c => c.universal_chain_id === universalChainId)
      if (chain) {
        return chain.display_name || chain.chain_id
      }
    }
    
    // Fallback to simple mapping if chains store not loaded
    const simpleNames: Record<string, string> = {
      'union-testnet-8': 'Union',
      'osmo-test-5': 'Osmosis', 
      'sepolia-1': 'Ethereum',
      'stride-internal-1': 'Stride'
    }
    return simpleNames[universalChainId] || universalChainId
  }
  
  const formatHash = (hash: string): string => {
    return hash ? `${hash.slice(0, 8)}...${hash.slice(-4)}` : 'N/A'
  }
  
  const formatAmount = (amount: string, token: string): string => {
    if (!amount || !token) return ''
    
    // Convert from base units (assuming 6 decimals for most tokens)
    const numAmount = parseFloat(amount) / 1_000_000
    const formatted = numAmount >= 1 ? numAmount.toFixed(2) : numAmount.toFixed(6)
    return `${formatted} ${token}`
  }
  


  // Check if transfer matches selected chain filter (for sound filtering)
  const shouldPlaySound = (transfer: any): boolean => {
    if (!selectedFromChain && !selectedToChain) return true
    
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
          const value = parseFloat(transfer.base_amount?.toString() || '0') || 1
          transactionAudio.playTransactionSound(value)
        }
        
        // Log all transfers (filtering happens in display)
        const sourceChain = getChainDisplayName(transfer.source_chain?.universal_chain_id || 'unknown')
        const destChain = getChainDisplayName(transfer.destination_chain?.universal_chain_id || 'unknown')
        const hash = formatHash(transfer.packet_hash)
        const amount = formatAmount(transfer.base_amount, transfer.base_token)
        
        addLog('transfer', 
          `${sourceChain} → ${destChain}`, 
          {
            hash,
            amount,
            sender: transfer.sender_canonical ? `${transfer.sender_canonical.slice(0, 8)}...${transfer.sender_canonical.slice(-4)}` : null,
            receiver: transfer.receiver_canonical ? `${transfer.receiver_canonical.slice(0, 8)}...${transfer.receiver_canonical.slice(-4)}` : null,
            timestamp: transfer.transfer_send_timestamp,
            sourceChainId: transfer.source_chain?.universal_chain_id,
            destChainId: transfer.destination_chain?.universal_chain_id
          }
        )
      })
      processedCount = transfers.length
    }
  })
  
  onMount(() => {
    addLog('system', 'Terminal log ready, watching for transfers...')
  })
  
  const clearLogs = () => {
    allLogs = []
    startTime = Date.now()
  }
</script>

<Card class="h-full flex flex-col">
  <div class="flex flex-col h-full text-zinc-300 font-mono text-sm overflow-hidden">
    {#if selectedFromChain || selectedToChain}
      <div class="px-2 py-1 bg-zinc-800 border-b border-zinc-700 text-xs text-amber-400">
        {#if selectedFromChain && selectedToChain}
          Filtering: {getChainDisplayName(selectedFromChain)} → {getChainDisplayName(selectedToChain)}
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
        {#if log.data}
          <div class="text-zinc-300 font-mono">
            {#if log.message}
            <div class="text-zinc-300 font-mono">{log.message}</div>
          {/if}
            {#if log.data.hash}
              <div class="text-zinc-300 font-mono">tx: {log.data.hash}</div>
            {/if}
            {#if log.data.sender}
              <div class="text-zinc-400">from: {log.data.sender}</div>
            {/if}
            {#if log.data.receiver}
              <div class="text-zinc-400">to: {log.data.receiver}</div>
            {/if}
          </div>
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