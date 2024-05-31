<script lang="ts">
import request from "graphql-request"
import { channelsQuery } from "$lib/graphql/documents/channels.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { writable } from "svelte/store"
import CellDurationText from "../(components)/cell-duration-text.svelte"
import CellStatus from "../(components)/cell-status.svelte"
import { DurationUnits } from "svelte-ux"

$: channels = createQuery({
  queryKey: ["channels"],
  refetchInterval: 5_000,
  queryFn: async () => request(URLS.GRAPHQL, channelsQuery, {})
})

$: channelsData = $channels?.data?.v0_channel_map ?? []

type Channel = (typeof channelsData)[number]

$: channelsStore = writable<Array<Channel>>(channelsData as Array<Channel>)
$: if (channels) {
  channelsStore.update(channels => channels)
}

const columns: Array<ColumnDef<{ chain_id: string }>> = [
  {
    accessorKey: "from_chain_id",
    header: () => "From Chain ID",
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
    accessorKey: "from_channel_id",
    header: () => "From Channel ID",
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
    accessorKey: "to_connection_id",
    header: () => "To Connection ID",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "to_channel_id",
    header: () => "To Channel ID",
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

<Table bind:dataStore={channelsStore} {columns} />
