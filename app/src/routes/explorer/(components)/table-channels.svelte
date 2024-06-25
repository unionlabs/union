<script lang="ts">
import request from "graphql-request"
import { channelsQuery } from "$lib/graphql/documents/channels.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import CellOriginChannel from "../(components)/cell-origin-channel.svelte"
import { raise } from "$lib/utilities"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { toDisplayName } from "$lib/utilities/chains.ts"
import type { Chain } from "$lib/types"

export let chains: Array<Chain>;

let channels = createQuery({
  queryKey: ["channels"],
  refetchInterval: 5_000,
  retryDelay: attempt => Math.min(attempt > 1 ? 2 ** attempt * 1000 : 1000, 30 * 1000), // expo backoff
  queryFn: async () => { 
    const response = await request(URLS.GRAPHQL, channelsQuery, {})
    if (!response.v0_channel_map) raise("error fetching transfers")

    return response.v0_channel_map.map(channel => ({
      source: {
        chain_display_name: toDisplayName(channel.from_chain_id, chains),
        chain_id: channel.from_chain_id ?? "unknown",
        connection_id: channel.to_connection_id ?? "unknown",
        channel_id: channel.from_channel_id ?? "unknown",
      },
      destination: {
        chain_display_name: toDisplayName(channel.from_chain_id, chains),
        chain_id: channel.to_chain_id ?? "unknown",
        connection_id: channel.to_connection_id ?? "unknown",
        channel_id: channel.to_channel_id ?? "unknown",
      },
      status: channel.status
    }));
  }
})

let channelsData = derived(channels, $channels => $channels.data ?? []);

const columns: Array<ColumnDef<{}>> = [
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

{#if $channels.data }
  <Table bind:dataStore={channelsData} {columns} />
{:else if $channels.isLoading}
  <LoadingLogo class="size-16"/>
{:else if $channels.isError}
  Error fetching channels...
{/if}
