<script lang="ts">
import request from "graphql-request"
import { channelsQuery } from "$lib/graphql/documents/channels.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import CellOriginChannel from "../(components)/cell-origin-channel.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/types"

let channels = createQuery({
  queryKey: ["channels"],
  refetchInterval: 5_000,
  retryDelay: attempt => Math.min(attempt > 1 ? 2 ** attempt * 1000 : 1000, 30 * 1000),
  queryFn: async () => request(URLS.GRAPHQL, channelsQuery, {}),
  select: data =>
    data.v0_channel_map.map(channel => ({
      source: {
        chain_display_name: channel.source?.display_name,
        chain_id: channel.from_chain_id ?? "unknown",
        connection_id: channel.to_connection_id ?? "unknown",
        channel_id: channel.from_channel_id ?? "unknown",
        port_id: channel.from_port_id ?? "unknown"
      },
      destination: {
        chain_display_name: channel.destination?.display_name,
        chain_id: channel.to_chain_id ?? "unknown",
        connection_id: channel.to_connection_id ?? "unknown",
        channel_id: channel.to_channel_id ?? "unknown",
        port_id: channel.to_port_id ?? "unknown"
      },
      status: channel.status
    }))
})

let channelsDataStore = derived(channels, $channels => $channels.data ?? [])

type DataRow = UnwrapReadable<typeof channelsDataStore>[number]

const columns: Array<ColumnDef<DataRow>> = [
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
  }
]
</script>

{#if $channels.data}
  <Table bind:dataStore={channelsDataStore} {columns} />
{:else if $channels.isLoading}
  <LoadingLogo class="size-16" />
{:else if $channels.isError}
  Error fetching channels...
{/if}
