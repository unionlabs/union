<script lang="ts">
import request from "graphql-request"
import { packetsQuery } from "$lib/graphql/documents/packets.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived, writable } from "svelte/store"
import CellStatus from "$lib/components/table-cells/cell-status.svelte"
import { DurationUnits } from "svelte-ux"
import CellOriginChannel from "$lib/components/table-cells/cell-origin-channel.svelte"
import CellDurationText from "$lib/components/table-cells/cell-duration-text.svelte"
import CellTimestamp from "$lib/components/table-cells/cell-timestamp.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types"

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
      source_time: packet.source_time,
      destination_time: packet.destination_time
    }))
})

let packetsDataStore = derived(packets, $packets => $packets.data ?? [])

type PacketRow = UnwrapReadable<typeof packetsDataStore>[number]

const columns: Array<ColumnDef<PacketRow>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    size: 200,
    cell: info => flexRender(CellOriginChannel, { value: info.getValue() })
  },
  {
    accessorKey: "destination",
    header: () => "Destination",
    size: 200,
    cell: info => flexRender(CellOriginChannel, { value: info.getValue() })
  },
  {
    header: () => "Source Time",
    accessorKey: "source_time",
    cell: info => flexRender(CellTimestamp, { value: info.getValue() })
  },
  {
    header: () => "Destination Time",
    accessorKey: "destination_time",
    cell: info => flexRender(CellTimestamp, { value: info.getValue() })
  }
]
</script>

{#if $packets.data}
  <Table bind:dataStore={packetsDataStore} {columns} />
{:else if $packets.isLoading}
  <LoadingLogo class="size-16" />
{:else if $packets.isError}
  Error fetching packets...
{/if}

