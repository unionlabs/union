<script lang="ts">
import request from "graphql-request"
import { URLS } from "$lib/constants"
import type { Chain } from "$lib/types"
import { clientHeightsQuery } from "$lib/graphql/queries/client-heights.ts"
import { createQuery } from "@tanstack/svelte-query"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { derived } from "svelte/store"

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
  <table class="center">
    <caption>clients</caption>
    <thead>
      <tr>
        <th colspan="2" rowspan="2"></th>
        <th colspan={length}>tracker</th>
      </tr>
      <tr>
        {#each Object.keys($tableClients) as chain}
          <th>{chain}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each Object.entries($tableClients) as [counterpartyChainId, _], counterpartyIndex (counterpartyChainId)}
        <tr>
          {#if counterpartyIndex === 0}
            <th rowspan={length}>
              <div>tracked</div>
            </th>
          {/if}
          <th>{counterpartyChainId}</th>
          {#each Object.entries($tableClients) as [chainId, values] (chainId)}
            {#if chainId === counterpartyChainId}
              <td class="bg-default"></td>
            {:else}
              {@const counterpartyChain = getCounterpartyChain($tableChains, counterpartyChainId)}
              {@const client = getClient(values, counterpartyChainId)}
              <td class="p-2">
                <div class="text-xs text-green-600 ">Client {client.client_id}</div>
                <div>D {counterpartyChain.index_status.height - client.max_counterparty_height}</div>
                <div>C {client.max_counterparty_height}</div>
                <div>I {counterpartyChain.index_status.height}</div>
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
