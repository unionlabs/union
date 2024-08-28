
<script lang="ts">
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import type { Chain } from "$lib/types.ts"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellTimestamp from "$lib/components/table-cells/cell-timestamp.svelte"
import CellSequence from "$lib/components/table-cells/cell-sequence.svelte"
import { derived, type Readable } from "svelte/store"
import CellOriginChannel from "$lib/components/table-cells/cell-origin-channel.svelte"

import ExplorerTablePaginated from "$lib/components/explorer-table-paginated.svelte"
import {
  packetsQuery,
  packetsByChainIdQuery,
  packetsByConnectionIdQuery,
  packetsByChannelIdQuery
} from "$lib/queries/packets"
import { timestamp } from "$lib/stores/page.ts"

// export let chains: Array<Chain>
export let chain_id: string | undefined = undefined
export let connection_id: string | undefined = undefined
export let channel_id: string | undefined = undefined
// export let pageSize: number // must be even

let packets = chain_id
  ? connection_id
    ? channel_id
      ? packetsByChannelIdQuery(12, chain_id, connection_id, channel_id, timestamp)
      : packetsByConnectionIdQuery(12, chain_id, connection_id, timestamp)
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
    header: () => "Sequence",
    accessorKey: "source_sequence",
    cell: info => flexRender(CellSequence, { value: info.getValue() })
  }
]
</script>

<ExplorerTablePaginated queryResult={packets} dataStore={packetsDataStore} {columns}/>
