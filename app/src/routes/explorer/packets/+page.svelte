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
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types"

const packets = createQuery({
  queryKey: ["packets"],
  refetchInterval: 5_000,
  queryFn: async () => request(URLS.GRAPHQL, packetsQuery, {}),
  select: data =>
    data.v0_packets.map(channel => ({
      source: {
        chain_id: channel.from_chain_id ?? "unknown",
        connection_id: channel.from_connection_id ?? "unknown",
        channel_id: channel.from_channel_id ?? "unknown",
        port_id: channel.from_port_id ?? "unknown"
      },
      destination: {
        chain_id: channel.to_chain_id ?? "unknown",
        connection_id: channel.to_connection_id ?? "unknown",
        channel_id: channel.to_channel_id ?? "unknown",
        port_id: channel.to_port_id ?? "unknown"
      },
      status: channel.status
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

{#if $packets.data}
  <Table bind:dataStore={packetsDataStore} {columns} />
{:else if $packets.isLoading}
  <LoadingLogo class="size-16" />
{:else if $packets.isError}
  Error fetching packets...
{/if}

