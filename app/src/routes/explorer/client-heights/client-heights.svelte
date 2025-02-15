<script lang="ts">
import request from "graphql-request"
import { URLS } from "$lib/constants"
import type { Chain } from "$lib/types"
import { clientHeightsQuery } from "$lib/graphql/queries/client-heights.ts"
import { createQuery } from "@tanstack/svelte-query"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { derived } from "svelte/store"
import ChainDetails from "$lib/chain-details.svelte"

export const clientHeights = createQuery({
  queryKey: ["client-heights"],
  enabled: true,
  refetchOnMount: true,
  refetchOnReconnect: true,
  refetchInterval: () => 5_000,
  queryFn: () => request(URLS().GRAPHQL, clientHeightsQuery, {})
})
export let chains: Array<Chain>

export let tableChains = derived(clientHeights, $clientHeights =>
  $clientHeights.data ? $clientHeights.data.v1_ibc_union_chains : null
)

export let tableClients = derived(clientHeights, $clientHeights =>
  $clientHeights.data
    ? Object.groupBy(
        $clientHeights.data.v1_ibc_union_client_heights_max,
        client => client.client_chain.chain_id
      )
    : null
)

function getCounterpartyChain(chains, counterpartyChainId) {
  return chains.find(chain => chain.index_status.chain_id === counterpartyChainId)
}

function getClient(clients, counterpartyChainId) {
  return clients.find(value => value.counterparty_chain.chain_id === counterpartyChainId)
}
</script>

{#if $clientHeights.data && $tableChains && $tableClients}
<div class="table-responsive">
  <table class="gap-x-2">
    <thead>
      <tr>
        <th colspan="2" rowspan="2"></th>
        <th colspan={Object.keys($tableClients).length} class="text-muted-foreground pb-4">tracker</th>
      </tr>
      <tr>
        {#each Object.keys($tableClients) as chain}
          <th class="pb-2 px-4"><ChainDetails {chains} chainId={chain}/></th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each Object.entries($tableClients) as [counterpartyChainId, _], counterpartyIndex (counterpartyChainId)}
        <tr>
          {#if counterpartyIndex === 0}
            <th rowspan={length}>
              <div class="text-muted-foreground -rotate-90">tracked</div>
            </th>
          {/if}
          <th class="text-right pr-4"><ChainDetails {chains} chainId={counterpartyChainId}/></th>
          {#each Object.entries($tableClients) as [chainId, values] (chainId)}
            {#if chainId === counterpartyChainId}
              <td></td>
            {:else}
              {@const counterpartyChain = getCounterpartyChain($tableChains, counterpartyChainId)}
              {@const client = getClient(values, counterpartyChainId)}
              <td class="p-2 bg-muted">
                <div class="text-xs text-union-accent-950 font-bold dark:text-union-accent ">Client {client.client_id}</div>
                <div><span class="text-muted-foreground italic">Δ</span> {counterpartyChain.index_status.height - client.max_counterparty_height}</div>
                <div><span class="text-muted-foreground italic">C</span> {client.max_counterparty_height}</div>
                <div><span class="text-muted-foreground italic">I</span> {counterpartyChain.index_status.height}</div>
              </td>
            {/if}
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>
  
{:else}
  <LoadingLogo/>
{/if}
