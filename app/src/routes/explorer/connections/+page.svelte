<script lang="ts">
import request from "graphql-request"
import { connectionsQuery } from "$lib/graphql/documents/connections.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import CellOriginConnection from "../(components)/cell-origin-connection.svelte"
import { raise } from "$lib/utilities"


let connections = createQuery({
  queryKey: ["connections"],
  refetchInterval: 5_000,
  queryFn: async () => { 
    const response = await request(URLS.GRAPHQL, connectionsQuery, {})
    if (!response.v0_connection_map) raise("error fetching transfers")

    return response.v0_connection_map.map(connection => ({
      source: {
        chain_display_name: connection.from_chain_id ?? "unknown",
        chain_id: connection.from_chain_id ?? "unknown",
        connection_id: connection.from_connection_id ?? "unknown",
        client_id: connection.from_client_id ?? "unknown",
      },
      destination: {
        chain_display_name: connection.to_chain_id ?? "unknown",
        chain_id: connection.to_chain_id ?? "unknown",
        connection_id: connection.to_connection_id ?? "unknown",
        client_id: connection.to_client_id ?? "unknown",
      },
      status: connection.status
    }));
  }
})

let connectionsData = derived(connections, ($connections) =>  $connections.data ?? [] )

const columns: Array<ColumnDef<{ chain_id: string }>> = [
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

{#if $connections.data }
  <Table bind:dataStore={connectionsData} {columns} />
{/if}
