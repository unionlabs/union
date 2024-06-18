<script lang="ts">
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { writable } from "svelte/store"
import Table from "../(components)/table.svelte"
import { createQuery } from "@tanstack/svelte-query"
import CellStatus from "../(components)/cell-status.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import CellDurationText from "../(components)/cell-duration-text.svelte"
import { indexStatusQuery } from "$lib/graphql/documents/index-status.ts"

$: indexStatus = createQuery({
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

$: indexStatusData = $indexStatus?.data ?? []

type IndexStatus = (typeof indexStatusData)[number]

$: indexStatusStore = writable<Array<IndexStatus>>(indexStatusData as Array<IndexStatus>)

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

<Table bind:dataStore={indexStatusStore} {columns} />
