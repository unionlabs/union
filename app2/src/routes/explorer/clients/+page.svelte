<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import { clientsQuery } from "$lib/queries/clients.svelte.ts"
import { runFork, runFork$ } from "$lib/runtime"
import { chains } from "$lib/stores/chains.svelte"
import { clientsStore } from "$lib/stores/clients.svelte"
import { Fiber, Option } from "effect"
import { onMount } from "svelte"

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

let fiber: Fiber.Fiber<any, any>

// Fetch clients data on mount
onMount(() => {
  runFork$(() => clientsQuery())
})

// Get client status between two chains
function getClientStatus(fromChainId: string, toChainId: string) {
  if (fromChainId === toChainId) {
    return null // No self-connections
  }

  return clientsStore.data.pipe(
    Option.map(clients => {
      // Find client from fromChain to toChain
      const client = clients.find(c =>
        c.universal_chain_id === fromChainId
        && c.counterparty_universal_chain_id === toChainId
      )
      return client || null
    }),
    Option.getOrElse(() => null),
  )
}

// Check if client has active status (has height)
function hasActiveStatus(client: any) {
  return client?.status && Option.isSome(client.status)
    ? Option.isSome(client.status.value.height)
    : false
}

// Generate tooltip content for client status
function getTooltipContent(
  fromChainId: string,
  toChainId: string,
  fromChainName: string,
  toChainName: string,
) {
  const client = getClientStatus(fromChainId, toChainId)

  if (!client) {
    return `${fromChainName} → ${toChainName}: No client found`
  }

  const statusInfo = []

  if (client.client_id) {
    statusInfo.push(`Client ID: ${client.client_id}`)
  }

  if (client.status && Option.isSome(client.status)) {
    const status = client.status.value

    if (Option.isSome(status.height)) {
      statusInfo.push(`Height: ${status.height.value}`)
    }

    if (Option.isSome(status.counterparty_height)) {
      statusInfo.push(`Counterparty Height: ${status.counterparty_height.value}`)
    }

    if (Option.isSome(status.timestamp)) {
      statusInfo.push(`Timestamp: ${new Date(status.timestamp.value).toLocaleString()}`)
    }
  }

  if (client.chain && Option.isSome(client.chain) && Option.isSome(client.chain.value.status)) {
    const chainStatus = client.chain.value.status.value
    if (Option.isSome(chainStatus.status)) {
      statusInfo.push(`Chain Status: ${chainStatus.status.value}`)
    }
  }

  if (
    client.counterparty_chain && Option.isSome(client.counterparty_chain)
    && Option.isSome(client.counterparty_chain.value.status)
  ) {
    const counterpartyStatus = client.counterparty_chain.value.status.value
    if (Option.isSome(counterpartyStatus.status)) {
      statusInfo.push(`Counterparty Status: ${counterpartyStatus.status.value}`)
    }
  }

  return `${fromChainName} → ${toChainName}:\n${statusInfo.join("\n")}`
}

// Generate diagonal delay for animation (from top-right and bottom-left corners toward center)
function getDiagonalDelay(fromIndex: number, toIndex: number): number {
  const totalRows = sortedChains.length
  const totalColumns = sortedChains.length

  // Calculate distance from top-right corner (0, totalColumns-1)
  const distanceFromTopRight = fromIndex + (totalColumns - 1 - toIndex)

  // Calculate distance from bottom-left corner (totalRows-1, 0)
  const distanceFromBottomLeft = (totalRows - 1 - fromIndex) + toIndex

  // Use the minimum distance (closest corner)
  const minDistance = Math.min(distanceFromTopRight, distanceFromBottomLeft)

  return minDistance * 20 // 20ms delay per diagonal level
}

// Generate delay for row labels (all at once)
function getRowLabelDelay(fromIndex: number): number {
  return 0 // All rows fade in simultaneously
}

// Generate delay for column labels (all at once)
function getColumnLabelDelay(toIndex: number): number {
  return 0 // All columns fade in simultaneously
}
</script>

{#if sortedChains.length === 0}
  <div class="flex items-center justify-center h-64">
    <div class="text-zinc-400">Loading chains...</div>
  </div>
{:else}
  {#if Option.isSome(clientsStore.error)}
    <ErrorComponent error={clientsStore.error.value} />
  {/if}
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
            {#each sortedChains as toChain, toIndex}
              <th class="top-0 sticky z-10 max-w-8 h-[160px] bg-zinc-900">
                <div class="h-[160px] pt-2 border-l border-zinc-800">
                  <div class="transform rotate-90 z-20">
                    <div
                      class="w-[160px] flex items-start justify-start pl-2 animate-fade-in"
                      style="animation-delay: {getColumnLabelDelay(toIndex)}ms;"
                    >
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
          {#each sortedChains as fromChain, fromIndex}
            <tr>
              <td class="sticky left-0 bg-zinc-900 z-10 min-w-[160px]">
                <div class="border-t border-zinc-800 flex items-center h-8 pl-2">
                  <div
                    class="animate-fade-in"
                    style="animation-delay: {getRowLabelDelay(fromIndex)}ms;"
                  >
                    <ChainComponent
                      chain={fromChain}
                      class="text-xs"
                    />
                  </div>
                </div>
              </td>
              {#each sortedChains as toChain, toIndex}
                <td class="border-zinc-800 p-0 w-8 h-8">
                  {#if fromChain.universal_chain_id === toChain.universal_chain_id}
                    <div
                      class="w-full h-full bg-zinc-900 animate-scale-in"
                      style="animation-delay: {getDiagonalDelay(fromIndex, toIndex)}ms;"
                    >
                    </div>
                  {:else}
                    {@const client = getClientStatus(
                fromChain.universal_chain_id,
                toChain.universal_chain_id,
              )}
                    {@const hasStatus = client && hasActiveStatus(client)}
                    <div
                      class="w-full h-full border-t-1 border-l-1 border-zinc-900 animate-scale-in {hasStatus ? 'bg-green-500' : 'bg-red-500'}"
                      style="animation-delay: {getDiagonalDelay(fromIndex, toIndex)}ms;"
                      title={getTooltipContent(
                        fromChain.universal_chain_id,
                        toChain.universal_chain_id,
                        fromChain.display_name,
                        toChain.display_name,
                      )}
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

  {#if Option.isSome(clientsStore.data)}
    <div class="mt-8 p-4 bg-zinc-900 border border-zinc-800 rounded-lg">
      <h2 class="text-lg font-semibold text-zinc-300 mb-4">Query Response</h2>
      <pre class="text-xs text-zinc-400 overflow-auto max-h-96 p-4 bg-zinc-950 rounded">
{JSON.stringify(Option.getOrElse(clientsStore.data, () => []), null, 2)}
      </pre>
    </div>
  {/if}
{/if}

<style>
/* Custom scrollbar styling */
:global(.overflow-auto::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

:global(.overflow-auto::-webkit-scrollbar-track) {
  background: #27272a;
}

:global(.overflow-auto::-webkit-scrollbar-thumb) {
  background: #52525b;
  border-radius: 4px;
}

:global(.overflow-auto::-webkit-scrollbar-thumb:hover) {
  background: #71717a;
}

/* Scale in animation for matrix cells */
@keyframes scale-in {
  0% {
    transform: scale(0);
    opacity: 0;
  }
  50% {
    transform: scale(1.1);
    opacity: 0.8;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

.animate-scale-in {
  animation: scale-in 0.6s ease-out forwards;
  transform: scale(0);
  opacity: 0;
}

/* Fade in animation for headers */
@keyframes fade-in {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}

.animate-fade-in {
  animation: fade-in 0.6s ease-out forwards;
  opacity: 0;
}
</style>
