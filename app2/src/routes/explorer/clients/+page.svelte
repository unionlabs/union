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

{#if sortedChains.length === 0}
  <div class="flex items-center justify-center h-64">
    <div class="text-zinc-400">Loading chains...</div>
  </div>
{:else}
  <div class="overflow-auto max-h-full">
    <div class="inline-block min-w-full">
      <table class="border-collapse">
        <thead>
          <tr class="">
            <th class="top-0 sticky left-0 bg-zinc-900 z-30 p-2 text-xs font-medium text-zinc-300">
              <div class="transform -rotate-45">
                Host — Tracking
              </div>
            </th>
            {#each sortedChains as toChain}
              <th class="top-0 sticky z-10 max-w-8 h-[160px] bg-zinc-900">
                <div class="h-[160px] pt-2 border-l border-zinc-800">
                  <div class="transform rotate-90 z-20">
                    <div class="w-[160px] flex items-start justify-start pl-2">
                      <ChainComponent
                        chain={toChain}
                        class="text-xs"
                      />
                    </div>
                  </div>
                </div>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each sortedChains as fromChain}
            <tr>
              <td class="sticky left-0 bg-zinc-900 z-10 min-w-[160px]">
                <div class="border-t border-zinc-800 flex items-center h-8 pl-2">
                  <ChainComponent
                    chain={fromChain}
                    class="text-xs"
                  />
                </div>
              </td>
              {#each sortedChains as toChain}
                <td class="border-zinc-800 p-0 w-8 h-8">
                  {#if fromChain.universal_chain_id === toChain.universal_chain_id}
                    <div class="w-full h-full bg-zinc-900"></div>
                  {:else}
                    <div
                      class="w-full h-full border-t-1 border-l-1 border-zinc-900 {getConnectionStatus(fromChain.universal_chain_id, toChain.universal_chain_id) ? 'bg-green-500' : 'bg-red-500'}"
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
{/if}

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
