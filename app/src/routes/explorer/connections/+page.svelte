<script lang="ts">
import request from "graphql-request"
import { connectionsQuery } from "$lib/graphql/documents/connections.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { writable } from "svelte/store"
import CellStatus from "../(components)/cell-status.svelte"

$: connections = createQuery({
  queryKey: ["connections"],
  refetchInterval: 5_000,
  queryFn: async () => request(URLS.GRAPHQL, connectionsQuery, {})
})

$: connectionsData = $connections?.data?.v0_connection_map ?? []

type Connection = (typeof connectionsData)[number]

$: connectionsStore = writable<Array<Connection>>(connectionsData as Array<Connection>)
$: if (connections) {
  connectionsStore.update(connections => connections)
}

const columns: Array<ColumnDef<{ chain_id: string }>> = [
  {
    accessorKey: "from_chain_id",
    header: () => "From Chain ID",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "from_client_id",
    header: () => "From Client ID",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "from_connection_id",
    header: () => "From Connection ID",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "to_chain_id",
    header: () => "To Chain ID",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "to_client_id",
    header: () => "To Client ID",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "to_connection_id",
    header: () => "To Connection ID",
    size: 200,
    cell: info => info.getValue()
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

<Table bind:dataStore={connectionsStore} {columns} />
