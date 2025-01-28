<script lang="ts">
import LoadingLogo from "$lib/components/loading-logo.svelte"
import request from "graphql-request"
import { derived } from "svelte/store"
import { URLS } from "$lib/constants"
import type { UnwrapReadable } from "$lib/utilities/types"
import Table from "../(components)/table.svelte"
import { createQuery } from "@tanstack/svelte-query"
import CellStatus from "$lib/components/table-cells//cell-status.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import CellDurationText from "$lib/components/table-cells//cell-duration-text.svelte"
import CellChainIndex from "$lib/components/table-cells//cell-chain-index.svelte"
import { indexStatusQuery } from "$lib/graphql/queries/index-status.ts"
import type { ChainFeature } from "$lib/types.ts"
import { page } from "$app/stores"

let indexStatus = createQuery({
  queryKey: ["index-status"],
  refetchInterval: 500,
  queryFn: async () => request(URLS().GRAPHQL, indexStatusQuery, {}),
  select: data => {
    const enabledChains = data.chains.flatMap(chain => chain.chain_id)
    return data.statuses
      .filter(status => status.chain_id && enabledChains.includes(status.chain_id))
      .map(s => ({ chain: { chain_display_name: s.display_name, chain_id: s.chain_id }, ...s }))
  }
})

let indexStatusDataStore = derived([indexStatus, page], ([$indexStatus, $page]) => {
  const enabledChainIds = $page.data.features
    .filter((chain: ChainFeature) => chain.features[0]?.index_status)
    .map((chain: ChainFeature) => chain.chain_id)

  return ($indexStatus.data ?? []).filter(status => enabledChainIds.includes(status.chain.chain_id))
})

type DataRow = UnwrapReadable<typeof indexStatusDataStore>[number]

const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "chain",
    header: () => "Chain",
    size: 200,
    cell: info => flexRender(CellChainIndex, { value: info.getValue() })
  },
  {
    accessorKey: "timestamp",
    header: () => "Latest block",
    size: 200,
    cell: info => flexRender(CellDurationText, { value: info.getValue() })
  },
  {
    accessorKey: "status",
    header: () => "Status",
    size: 200,
    cell: info =>
      flexRender(CellStatus, {
        value: info.getValue()
      })
  }
]
</script>


{#if $indexStatus.data }
  <Table bind:dataStore={indexStatusDataStore} {columns} />
{:else if $indexStatus.isLoading}
  <LoadingLogo class="size-16" />
{:else if $indexStatus.isError}
  Error fetching index status...
{/if}
