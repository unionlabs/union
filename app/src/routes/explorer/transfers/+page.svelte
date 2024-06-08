<script lang="ts">
import request from "graphql-request"
import { allTransfersQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived, writable } from "svelte/store"
import CellStatus from "../(components)/cell-status.svelte"
import { DurationUnits } from "svelte-ux"
import CellDurationText from "../(components)/cell-duration-text.svelte"

let transfers = createQuery({
  queryKey: ["transfers"],
  refetchInterval: 1_000,
  queryFn: async () => (await request(URLS.GRAPHQL, allTransfersQueryDocument, {})).v0_transfers
})

let transfersData = derived(transfers, $transfers => ($transfers.isSuccess ? $transfers.data : []))

const columns: Array<ColumnDef<{ chain_id: string }>> = [
  {
    accessorKey: "source_chain_id",
    header: () => "Source Chain",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "source_channel_id",
    header: () => "Source Channel",
    size: 200,
    cell: info => info.getValue()
  },
  // {
  //   accessorKey: "source_port",
  //   header: () => "Source Port",
  //   size: 200,
  //   cell: info => info.getValue()
  // },
  {
    accessorKey: "destination_chain_id",
    header: () => "Destination Chain",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "destination_channel_id",
    header: () => "Destination Channel",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "assets",
    header: () => "Assets",
    size: 200,
    cell: info => JSON.stringify(info.getValue())
  },
  {
    accessorKey: "source_timestamp",
    header: () => "Source Time",
    size: 200,
    cell: info =>
      flexRender(CellDurationText, {
        totalUnits: 3,
        variant: "short",
        minUnits: DurationUnits.Second,
        start: new Date(info.getValue() as string)
      })
  },
  {
    accessorKey: "destination_timestamp",
    header: () => "Destination Time",
    size: 200,
    cell: info =>
      flexRender(CellDurationText, {
        totalUnits: 3,
        variant: "short",
        minUnits: DurationUnits.Second,
        start: new Date(info.getValue() as string)
      })
  }
]
</script>

<Table bind:dataStore={transfersData} {columns} />
