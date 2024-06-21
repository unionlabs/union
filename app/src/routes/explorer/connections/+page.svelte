<script lang="ts">
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { writable } from "svelte/store"
import Table from "../(components)/table.svelte"
import { createQuery } from "@tanstack/svelte-query"
import { rankItem } from "@tanstack/match-sorter-utils"
import CellStatus from "../(components)/cell-status.svelte"
import { connectionsQuery } from "$lib/graphql/documents/connections.ts"
import { flexRender, type ColumnDef, type FilterFn } from "@tanstack/svelte-table"

$: connections = createQuery({
  queryKey: ["connections"],
  refetchInterval: 5_000,
  queryFn: async () => request(URLS.GRAPHQL, connectionsQuery, {})
})

$: connectionsData = $connections?.data?.v0_connection_map ?? []

type DataRow = (typeof connectionsData)[number]

$: connectionsStore = writable<Array<DataRow>>(connectionsData as Array<DataRow>)
$: if (connections) {
  connectionsStore.update(connections => connections)
}

let globalFilter = ""
const fuzzyFilter = ((row, columnId, value, addMeta) => {
  const itemRank = rankItem(row.getValue(columnId), value)
  addMeta({ itemRank })
  return itemRank.passed
}) satisfies FilterFn<DataRow>

const columns: Array<ColumnDef<DataRow>> = [
  {
    size: 200,
    accessorKey: "from_chain_id",
    header: () => "From Chain ID",
    filterFn: "includesString",
    accessorFn: row => row.from_chain_id,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "from_client_id",
    header: () => "From Client ID",
    filterFn: "includesString",
    accessorFn: row => row.from_client_id,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "from_connection_id",
    header: () => "From DataRow ID",
    filterFn: "includesString",
    accessorFn: row => row.from_connection_id,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "to_chain_id",
    header: () => "To Chain ID",
    filterFn: "includesString",
    accessorFn: row => row.to_chain_id,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "to_client_id",
    header: () => "To Client ID",
    filterFn: "includesString",
    accessorFn: row => row.to_client_id,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "to_connection_id",
    header: () => "To DataRow ID",
    filterFn: "includesString",
    accessorFn: row => row.to_connection_id,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "status",
    header: () => "Status",
    filterFn: "includesString",
    accessorFn: row => row.status,
    cell: info =>
      flexRender(CellStatus, {
        value: info.getValue()
      })
  }
]
</script>

<Table
  {columns}
  {fuzzyFilter}
  {globalFilter}
  enableFiltering={true}
  tableName="Connections"
  bind:dataStore={connectionsStore}
/>
