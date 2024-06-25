<script lang="ts">
import LoadingLogo from "$lib/components/loading-logo.svelte"
import request from "graphql-request"
import { derived } from "svelte/store"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { createQuery } from "@tanstack/svelte-query"
import CellStatus from "../(components)/cell-status.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import CellDurationText from "../(components)/cell-duration-text.svelte"
import { indexStatusQuery } from "$lib/graphql/documents/index-status.ts"

let indexStatus = createQuery({
  queryKey: ["index-status"],
  refetchInterval: 500,
  queryFn: async () => request(URLS.GRAPHQL, indexStatusQuery, {}),
  select: data => {
    const enabledChains = data.chains.flatMap(chain => chain.chain_id)
    return data.statuses.filter(
      status => status.chain_id && enabledChains.includes(status.chain_id)
    )
  }
})

let indexStatusData = derived(indexStatus, $indexStatus => $indexStatus.data ?? []);

const columns: Array<ColumnDef<{ chain_id: string }>> = [
  {
    accessorKey: "display_name",
    header: () => "Chain",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "chain_id",
    header: () => "Chain ID",
    size: 200,
    cell: info => info.getValue()
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
  <Table bind:dataStore={indexStatusData} {columns} />
{:else if $indexStatus.isLoading}
  <LoadingLogo class="size-16"/>
{:else if $indexStatus.isError}
  Error fetching index status...
{/if}
