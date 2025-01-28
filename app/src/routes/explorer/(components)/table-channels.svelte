<script lang="ts">
import request from "graphql-request"
import { channelsQuery } from "$lib/graphql/queries/channels.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import CellOriginChannel from "$lib/components/table-cells/cell-origin-channel.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types"
import type { ChainFeature } from "$lib/types.ts"
import { page } from "$app/stores"

let channels = createQuery({
  queryKey: ["channels"],
  refetchInterval: 5_000,
  retryDelay: attempt => Math.min(attempt > 1 ? 2 ** attempt * 1000 : 1000, 30 * 1000),
  queryFn: async () => request(URLS().GRAPHQL, channelsQuery, {}),
  select: data =>
    data.v1_ibc_union_channels.map(channel => ({
      source_chain: {
        chain_display_name: channel.destination_chain?.display_name ?? "unknown",
        chain_id: channel.source_chain_id ?? "unknown",
        connection_id: channel.destination_connection_id ?? "unknown",
        channel_id: channel.source_channel_id ?? "unknown",
        port_id: channel.source_port_id ?? "unknown"
      },
      destination_chain: {
        chain_display_name: channel.source_chain?.display_name ?? "unknown",
        chain_id: channel.destination_chain_id ?? "unknown",
        connection_id: channel.destination_connection_id ?? "unknown",
        channel_id: channel.destination_channel_id ?? "unknown",
        port_id: channel.destination_port_id ?? "unknown"
      },
      status: channel.status
    }))
})

let channelsDataStore = derived([channels, page], ([$channels, $page]) => {
  const enabledChainIds = $page.data.features
    .filter((chain: ChainFeature) => chain.features[0]?.channel_list)
    .map((chain: ChainFeature) => chain.chain_id)

  return ($channels.data ?? []).filter(
    channel =>
      enabledChainIds.includes(channel.source_chain.chain_id) &&
      enabledChainIds.includes(channel.destination_chain.chain_id)
  )
})

type DataRow = UnwrapReadable<typeof channelsDataStore>[number]

const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "source_chain",
    header: () => "Source chain",
    size: 200,
    cell: info => flexRender(CellOriginChannel, { value: info.getValue() })
  },
  {
    accessorKey: "destination_chain",
    header: () => "Destination chain",
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
