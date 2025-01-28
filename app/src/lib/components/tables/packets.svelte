<script lang="ts">
import type { ColumnDef } from "@tanstack/table-core"
import { flexRender } from "@tanstack/svelte-table"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
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
import CellCopy from "../table-cells/cell-copy.svelte"
import { page } from "$app/stores"
import type { ChainFeature } from "$lib/types.ts"

// export let chains: Array<Chain>
export let chain_id: string | undefined = undefined
export let connection_id: number | undefined = undefined
export let channel_id: number | undefined = undefined
// export let pageSize: number // must be even

let packets = chain_id
  ? connection_id
    ? channel_id
      ? packetsByChannelIdQuery(12, chain_id, connection_id, channel_id, timestamp)
      : packetsByConnectionIdQuery(12, chain_id, connection_id, timestamp)
    : packetsByChainIdQuery(12, chain_id, timestamp)
  : packetsQuery(12, timestamp)

let packetsDataStore = derived([packets, page], ([$packets, $page]) =>
  ($packets.data ?? [])
    .filter(packet => {
      const enabledChainIds = $page.data.features
        .filter((chain: ChainFeature) => chain.features[0]?.packet_list)
        .map((chain: ChainFeature) => chain.chain_id)

      return (
        enabledChainIds.includes(packet.source.chain_id) &&
        enabledChainIds.includes(packet.destination.chain_id)
      )
    })
    .map(item => ({
      ...item,
      timestamp: item.timestamp?.toString() ?? ""
    }))
)

type PacketRow = UnwrapReadable<typeof packetsDataStore>[number] & { timestamp?: string }

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
    accessorKey: "send",
    cell: info => flexRender(CellSequence, { value: info.getValue() })
  },
  {
    accessorKey: "recv",
    cell: info => flexRender(CellSequence, { value: info.getValue() })
  }
]
</script>

<ExplorerTablePaginated queryResult={packets} dataStore={packetsDataStore} {columns}/>
