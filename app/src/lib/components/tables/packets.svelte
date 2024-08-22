
<script lang="ts">
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import type { Chain } from "$lib/types.ts"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellTimestamp from "$lib/components/table-cells/cell-timestamp.svelte"
import { derived, type Readable } from "svelte/store"
import CellOriginChannel from "$lib/components/table-cells/cell-origin-channel.svelte"

import ExplorerTablePaginated from "$lib/components/explorer-table-paginated.svelte"
import { packetsQuery, packetsByChainIdQuery, packetsByConnectionIdQuery } from "$lib/queries/packets"
import { timestamp } from "$lib/stores/page.ts"

// export let chains: Array<Chain>
export let chain_id: string | undefined = undefined
export let connection_id: string | undefined = undefined
// export let pageSize: number // must be even

let packets = chain_id
  ? connection_id ? packetsByConnectionIdQuery(12, chain_id, connection_id, timestamp)
    : packetsByChainIdQuery(12, chain_id, timestamp)
  : packetsQuery(12, timestamp)
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
