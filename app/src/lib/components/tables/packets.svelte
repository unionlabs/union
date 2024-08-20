
<script lang="ts">
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import type { Chain } from "$lib/types.ts"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellTimestamp from "$lib/components/table-cells/cell-timestamp.svelte"
import { derived } from "svelte/store"
import CellOriginChannel from "$lib/components/table-cells/cell-origin-channel.svelte"

import ExplorerTablePaginated from "$lib/components/explorer-table-paginated.svelte"
import { createQuery } from "@tanstack/svelte-query"
import request from "graphql-request"
import { packetsQuery } from "$lib/graphql/queries/packets"

// export let chains: Array<Chain>
// export let pageSize: number // must be even

const packets = createQuery({
  queryKey: ["packets"],
  refetchInterval: 5_000,
  queryFn: async () => request(URLS.GRAPHQL, packetsQuery, {}),
  select: data =>
    data.v0_packets.map(packet => ({
      source: {
        chain_id: packet.from_chain_id ?? "unknown",
        connection_id: packet.from_connection_id ?? "unknown",
        channel_id: packet.from_channel_id ?? "unknown",
        port_id: packet.from_port_id ?? "unknown"
      },
      destination: {
        chain_id: packet.to_chain_id ?? "unknown",
        connection_id: packet.to_connection_id ?? "unknown",
        channel_id: packet.to_channel_id ?? "unknown",
        port_id: packet.to_port_id ?? "unknown"
      },
      timestamp: packet.source_time,
      destination_time: packet.destination_time
    }))
})

let packetsDataStore = derived(packets, $packets => $packets.data ?? [])

type PacketRow = UnwrapReadable<typeof packetsDataStore>[number]

const columns: Array<ColumnDef<PacketRow>> = [
  {
    accessorKey: "source",
    size: 200,
    cell: info => flexRender(CellOriginChannel, { value: info.getValue() })
  },
  {
    accessorKey: "destination",
    size: 200,
    cell: info => flexRender(CellOriginChannel, { value: info.getValue() })
  },
  {
    header: () => "Source Time",
    accessorKey: "timestamp",
    cell: info => flexRender(CellTimestamp, { value: info.getValue() })
  },
  {
    header: () => "Destination Time",
    accessorKey: "destination_time",
    cell: info => flexRender(CellTimestamp, { value: info.getValue() })
  }
]
</script>

<ExplorerTablePaginated queryResult={packets} dataStore={packetsDataStore} {columns}/>
