<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import Switch from "$lib/components/ui/Switch.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import { clientsQuery } from "$lib/queries/clients.svelte.ts"
import { runFork, runFork$ } from "$lib/runtime"
import { fetchFinalizedHeights } from "$lib/services/voyager-rpc"
import { chains } from "$lib/stores/chains.svelte"
import { clientsStore } from "$lib/stores/clients.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import { Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"

// Get chains from the store, separated by testnet/mainnet and sorted alphabetically
const mainnetChains = $derived(
  chains.data.pipe(
    Option.map(chainsData =>
      chainsData
        .filter(chain => !chain.testnet)
        .slice()
        .sort((a, b) => a.universal_chain_id.localeCompare(b.universal_chain_id))
    ),
    Option.getOrElse(() => []),
  ),
)

const testnetChains = $derived(
  chains.data.pipe(
    Option.map(chainsData =>
      chainsData
        .filter(chain => chain.testnet)
        .slice()
        .sort((a, b) => a.universal_chain_id.localeCompare(b.universal_chain_id))
    ),
    Option.getOrElse(() => []),
  ),
)

let fiber: Fiber.Fiber<any, any>

// Store for finalized heights
let finalizedHeights = $state<Map<string, Option.Option<string>>>(new Map())

// Get current chains based on mainnetOnly setting
const currentChains = $derived(settingsStore.mainnetOnly ? mainnetChains : testnetChains)

// Fetch clients data and finalized heights on mount
onMount(() => {
  runFork$(() => clientsQuery())

  // Fetch finalized heights when chains are available
  const allChains = [...mainnetChains, ...testnetChains]
  if (allChains.length > 0) {
    const chainIds = allChains.map(chain => chain.universal_chain_id)
    runFork$(() =>
      fetchFinalizedHeights(chainIds).pipe(
        Effect.map(heights => {
          finalizedHeights = heights
        }),
      )
    )
  }
})

// Re-fetch heights when chains change
$effect(() => {
  const allChains = [...mainnetChains, ...testnetChains]
  if (allChains.length > 0) {
    const chainIds = allChains.map(chain => chain.universal_chain_id)
    runFork$(() =>
      fetchFinalizedHeights(chainIds).pipe(
        Effect.map(heights => {
          finalizedHeights = heights
        }),
      )
    )
  }
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

// Generate tooltip data for client status
function getTooltipData(fromChainId: string, toChainId: string) {
  const client = getClientStatus(fromChainId, toChainId)
  const trackedChainHeight = finalizedHeights.get(toChainId)
  return {
    client,
    hasClient: !!client,
    trackedChainHeight,
  }
}

// Generate diagonal delay for animation (from top-right and bottom-left corners toward center)
function getDiagonalDelay(fromIndex: number, toIndex: number, chainsArray: any[]): number {
  const totalRows = chainsArray.length
  const totalColumns = chainsArray.length

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

{#snippet matrixCell(
  fromChain: any,
  toChain: any,
  fromIndex: number,
  toIndex: number,
  chainsArray: any[],
)}
  <td class="border-zinc-900 p-0 w-8 h-8">
    {#if fromChain.universal_chain_id === toChain.universal_chain_id}
      <div
        class="w-8 h-8 animate-scale-in border-t border-l border-zinc-900 bg-zinc-925"
        style="animation-delay: {getDiagonalDelay(fromIndex, toIndex, chainsArray)}ms;"
      >
      </div>
    {:else}
      {@const client = getClientStatus(
      fromChain.universal_chain_id,
      toChain.universal_chain_id,
    )}
      {@const hasStatus = client && hasActiveStatus(client)}
      {@const tooltipData = getTooltipData(
      fromChain.universal_chain_id,
      toChain.universal_chain_id,
    )}

      <Tooltip>
        {#snippet trigger()}
          <div
            class="w-8 h-8 animate-scale-in border-t border-l border-zinc-900 {hasStatus ? 'bg-accent' : 'bg-zinc-925'}"
            style="animation-delay: {getDiagonalDelay(fromIndex, toIndex, chainsArray)}ms;"
          >
          </div>
        {/snippet}

        {#snippet content()}
          <section>
            <div class="flex gap-2 items-center text-lg text-white font-bold">
              <div>{fromChain.display_name}</div>
              <div>→</div>
              <div>{toChain.display_name}</div>
            </div>
          </section>

          {#if !tooltipData.hasClient}
            <section>
              <Label>Status</Label>
              <div class="text-red-400">No client found</div>
            </section>
          {:else if tooltipData.client}
            {@const clientData = tooltipData.client}

            <section>
              <Label>Client ID</Label>
              <LongMonoWord>{clientData.client_id}</LongMonoWord>
            </section>
            {#if clientData.status && Option.isSome(clientData.status)}
              {@const status = clientData.status.value}

              {#if Option.isSome(status.height)}
                <section>
                  <Label>Host Height</Label>
                  <LongMonoWord>{status.height.value}</LongMonoWord>
                </section>
              {/if}

              {#if Option.isSome(status.counterparty_height)}
                <section>
                  <Label>Client Height</Label>
                  <LongMonoWord>{status.counterparty_height.value}</LongMonoWord>
                </section>
              {/if}

              {#if tooltipData.trackedChainHeight
        && Option.isSome(tooltipData.trackedChainHeight)}
                <section>
                  <Label>Counterparty Height</Label>
                  <LongMonoWord>{tooltipData.trackedChainHeight.value}</LongMonoWord>
                </section>
              {/if}

              {#if tooltipData.trackedChainHeight
        && Option.isSome(tooltipData.trackedChainHeight)
        && Option.isSome(status.counterparty_height)}
                {@const trackedHeight = parseInt(tooltipData.trackedChainHeight.value)}
                {@const counterpartyHeight = parseInt(status.counterparty_height.value.toString())}
                {@const delta = trackedHeight - counterpartyHeight}
                <section>
                  <Label>Client Height Delta</Label>
                  <LongMonoWord
                    class={delta < 0
                    ? "text-red-400"
                    : delta > 0
                    ? "text-green-400"
                    : "text-zinc-400"}
                  >
                    {delta > 0 ? "+" : ""}{delta}
                  </LongMonoWord>
                </section>
              {/if}

              {#if Option.isSome(status.timestamp)}
                <section>
                  <Label>Last Updated</Label>
                  <DateTimeComponent
                    class="text-sm hidden sm:block"
                    value={status.timestamp.value}
                  />
                </section>
              {/if}
            {/if}
          {/if}
        {/snippet}
      </Tooltip>
    {/if}
  </td>
{/snippet}

{#snippet matrixTable(chains: any[])}
  <div class="overflow-auto max-h-full">
    <div class="inline-block min-w-full">
      <table class="border-collapse">
        <thead>
          <tr class="">
            <th class="top-0 sticky left-0 bg-zinc-925 z-30 p-2 text-xs font-medium text-zinc-300">
              <div class="flex items-center justify-center h-full">
                <div class="transform -rotate-45 flex flex-col items-center gap-2">
                  <div class="text-sm font-medium">Host — Tracking</div>
                  <Switch
                    checked={settingsStore.mainnetOnly}
                    label={settingsStore.mainnetOnly ? "Mainnet" : "Testnet"}
                    change={(value) => settingsStore.mainnetOnly = value}
                    class="text-xs scale-75"
                  />
                </div>
              </div>
            </th>
            {#each chains as toChain, toIndex}
              <th class="top-0 sticky z-10 max-w-8 h-[160px] bg-zinc-925">
                <div class="h-[160px] pt-2 border-l border-zinc-900">
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
          {#each chains as fromChain, fromIndex}
            <tr>
              <td class="sticky left-0 bg-zinc-925 z-10 min-w-[160px]">
                <div class="border-t border-zinc-900 flex items-center h-8 pl-2">
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
              {#each chains as toChain, toIndex}
                {@render matrixCell(fromChain, toChain, fromIndex, toIndex, chains)}
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
{/snippet}

{#if currentChains.length === 0}
  <div class="flex items-center justify-center h-64">
    <div class="text-zinc-400">
      {#if settingsStore.mainnetOnly}
        No mainnet chains available
      {:else}
        No testnet chains available
      {/if}
    </div>
  </div>
{:else}
  {#if Option.isSome(clientsStore.error)}
    <ErrorComponent error={clientsStore.error.value} />
  {/if}

  {@render matrixTable(currentChains)}
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
