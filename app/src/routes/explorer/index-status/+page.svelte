<script lang="ts">
import request from "graphql-request"
import { indexStatusQuery } from "$lib/graphql/documents/index-status.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { removeArrayDuplicates } from "$lib/utilities"
import { writable } from "svelte/store"
import CellDurationText from "../(components)/cell-duration-text.svelte"
import CellStatus from "../(components)/cell-status.svelte"
import { DurationUnits } from "svelte-ux"

$: indexStatus = createQuery({
  queryKey: ["index-status"],
  refetchInterval: 500,
  queryFn: async () => request(URLS.GRAPHQL, indexStatusQuery, {})
})

$: indexStatusData = $indexStatus?.data?.v0_index_status ?? []

type IndexStatus = (typeof indexStatusData)[number]

$: indexStatusStore = writable<Array<IndexStatus>>(indexStatusData as Array<IndexStatus>)
$: if (indexStatus) {
  indexStatusStore.update(currentStatuses =>
    removeArrayDuplicates(
      [...(indexStatusData as Array<IndexStatus>), ...currentStatuses],
      "chain_id"
    )
  )
}

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
