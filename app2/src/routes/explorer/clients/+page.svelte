<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"

// Get chains from the store, sorted alphabetically by universal chain ID
const sortedChains = $derived(
  chains.data.pipe(
    Option.map(chainsData =>
      chainsData
        .slice()
        .sort((a, b) => a.universal_chain_id.localeCompare(b.universal_chain_id))
    ),
    Option.getOrElse(() => []),
  ),
)

// Generate random connection status for each chain pair
function getConnectionStatus(fromChain: string, toChain: string): boolean {
  if (fromChain === toChain) {
    return false // No self-connections
  }

  // Use a deterministic random based on chain names for consistency
  const seed = fromChain + toChain
  let hash = 0
  for (let i = 0; i < seed.length; i++) {
    const char = seed.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // Convert to 32bit integer
  }

  return Math.abs(hash) % 2 === 0 // Random true/false
}
</script>

<div class="p-6">
  <h1 class="text-2xl font-bold mb-6 text-white">Client Connections Matrix</h1>

  {#if sortedChains.length === 0}
    <div class="flex items-center justify-center h-64">
      <div class="text-gray-400">Loading chains...</div>
    </div>
  {:else}
    <div class="overflow-auto max-h-screen">
      <div class="inline-block min-w-full">
        <table class="border-collapse">
          <thead>
            <tr>
              <th class="sticky top-0 left-0 bg-gray-800 z-20 p-2 border border-gray-600 text-xs font-medium text-gray-300 min-w-[200px]">
                From / To
              </th>
              {#each sortedChains as toChain}
                <th class="sticky top-0 bg-gray-800 z-10 p-1 border border-gray-600 min-w-[60px] max-w-[60px]">
                  <div class="transform -rotate-90 origin-center whitespace-nowrap h-[200px] flex items-center justify-center">
                    <ChainComponent
                      chain={toChain}
                      class="text-xs"
                    />
                  </div>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each sortedChains as fromChain}
              <tr>
                <td class="sticky left-0 bg-gray-800 z-10 p-2 border border-gray-600 min-w-[200px]">
                  <ChainComponent
                    chain={fromChain}
                    class="text-xs"
                  />
                </td>
                {#each sortedChains as toChain}
                  <td class="border border-gray-600 p-0 w-15 h-15">
                    {#if fromChain.universal_chain_id === toChain.universal_chain_id}
                      <div class="w-full h-full bg-gray-700"></div>
                    {:else}
                      <div
                        class="w-full h-full {getConnectionStatus(fromChain.universal_chain_id, toChain.universal_chain_id) ? 'bg-green-500' : 'bg-red-500'}"
                        title="{fromChain.display_name} → {toChain.display_name}: {getConnectionStatus(fromChain.universal_chain_id, toChain.universal_chain_id) ? 'Connected' : 'Disconnected'}"
                      >
                      </div>
                    {/if}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

    <div class="mt-6 flex items-center gap-4 text-sm text-gray-300">
      <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-green-500"></div>
        <span>Connected</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-red-500"></div>
        <span>Disconnected</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-gray-700"></div>
        <span>Self (no connection)</span>
      </div>
    </div>
  {/if}
</div>

<style>
/* Custom scrollbar styling */
:global(.overflow-auto::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

:global(.overflow-auto::-webkit-scrollbar-track) {
  background: #374151;
}

:global(.overflow-auto::-webkit-scrollbar-thumb) {
  background: #6b7280;
  border-radius: 4px;
}

:global(.overflow-auto::-webkit-scrollbar-thumb:hover) {
  background: #9ca3af;
}
</style>
