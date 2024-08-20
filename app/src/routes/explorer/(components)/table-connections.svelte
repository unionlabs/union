<script lang="ts">
import request from "graphql-request"
import { connectionsQuery } from "$lib/graphql/queries/connections.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import CellOriginConnection from "$lib/components/table-cells/cell-origin-connection.svelte"
import { raise } from "$lib/utilities"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types.ts"

let connections = createQuery({
  queryKey: ["connections"],
  refetchInterval: 5_000,
  retryDelay: attempt => Math.min(attempt > 1 ? 2 ** attempt * 1000 : 1000, 30 * 1000), // expo backoff
  queryFn: async () => request(URLS.GRAPHQL, connectionsQuery, {}),
  select: data => {
    if (!data.v0_connection_map) raise("error fetching transfers")

    return data.v0_connection_map.map(connection => ({
      source: {
        chain_display_name: connection.source_chain?.display_name,
        chain_id: connection.from_chain_id ?? "unknown",
        connection_id: connection.from_connection_id ?? "unknown",
        client_id: connection.from_client_id ?? "unknown"
      },
      destination: {
        chain_display_name: connection.destination_chain?.display_name,
        chain_id: connection.to_chain_id ?? "unknown",
        connection_id: connection.to_connection_id ?? "unknown",
        client_id: connection.to_client_id ?? "unknown"
      },
      status: connection.status
    }))
  }
})

let connectionsDataStore = derived(connections, $connections => $connections.data ?? [])

type DataRow = UnwrapReadable<typeof connectionsDataStore>[number]

const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    size: 200,
    cell: info => flexRender(CellOriginConnection, { value: info.getValue() })
  },
  {
    accessorKey: "destination",
    header: () => "Destination",
    size: 200,
    cell: info => flexRender(CellOriginConnection, { value: info.getValue() })
  }
]
</script>

{#if $connections.data}
  <Table bind:dataStore={connectionsDataStore} {columns} />
{:else if $connections.isLoading}
  <LoadingLogo class="size-16" />
{:else if $connections.isError}
  Error fetching connections...
{/if}
