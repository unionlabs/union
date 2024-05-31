<script lang="ts">
import request from "graphql-request"
import { packetsQuery } from "$lib/graphql/documents/packets.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { writable } from "svelte/store"
import CellStatus from "../(components)/cell-status.svelte"
import { DurationUnits } from "svelte-ux"
import CellDurationText from "../(components)/cell-duration-text.svelte"

$: packets = createQuery({
  queryKey: ["packets"],
  refetchInterval: 5_000,
  queryFn: async () => request(URLS.GRAPHQL, packetsQuery, {})
})

$: packetsData = $packets?.data?.v0_packets ?? []

type Packet = (typeof packetsData)[number]

$: packetsStore = writable<Array<Packet>>(packetsData as Array<Packet>)
$: if (packets) {
  packetsStore.update(packets => packets)
}

const columns: Array<ColumnDef<{ chain_id: string }>> = [
  {
    accessorKey: "from_chain_id",
    header: () => "From Chain",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "from_channel_id",
    header: () => "From Channel",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "source_port",
    header: () => "From Port",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "to_chain_id",
    header: () => "To Chain",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "to_channel_id",
    header: () => "To Channel",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "to_port_id",
    header: () => "To Port",
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
  },
  {
    accessorKey: "source_time",
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
    accessorKey: "destination_time",
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

<Table bind:dataStore={packetsStore} {columns} />
