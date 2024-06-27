<script lang="ts">
import request from "graphql-request"
import { userTransfersQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import CellOrigin from "../(components)/cell-origin.svelte"
import CellAssets from "../(components)/cell-assets.svelte"
import { chainsQuery } from "$lib/queries/chains"
import { truncate } from "$lib/utilities/format"
import { cosmosStore } from "$lib/wallet/cosmos"
import { rawToHex } from "$lib/utilities/address"
import { sepoliaStore } from "$lib/wallet/evm"
import LoadingLogo from "$lib/components/loading-logo.svelte"

let transfers = createQuery({
  queryKey: ["user-transfers"],
  refetchInterval: 3_000,
  queryFn: async () => {
    const cosmosAddr = $cosmosStore?.rawAddress
    const evmAddr = $sepoliaStore?.address
    if (cosmosAddr === undefined || evmAddr === undefined) {
      return []
    }

    return (
      await request(URLS.GRAPHQL, userTransfersQueryDocument, {
        addr1: rawToHex(cosmosAddr).slice(2), // remove 0x
        addr2: evmAddr.slice(2) // remove 0x
      })
    ).v0_transfers
  }
})

let chains = chainsQuery()

let transfersData = derived([transfers, chains], ([$transfers, $chains]) => {
  if (!($transfers.isSuccess && $chains.isSuccess)) return []
  return $transfers.data.map(transfer => {
    const sourceDisplayName = $chains.data.find(
      chain => chain.chain_id === transfer.source_chain_id
    )?.display_name
    const destinationDisplayName = $chains.data.find(
      chain => chain.chain_id === transfer.destination_chain_id
    )?.display_name

    return {
      source: {
        name: sourceDisplayName,
        chain: transfer.source_chain_id,
        connection: transfer.source_connection_id,
        channel: transfer.source_channel_id,
        timestamp: transfer.source_timestamp
      },

      sender: transfer.sender,

      destination: {
        name: destinationDisplayName,
        chain: transfer.destination_chain_id,
        connection: transfer.destination_connection_id,
        channel: transfer.destination_channel_id,
        timestamp: transfer.destination_timestamp
      },

      receiver: transfer.receiver,
      assets: transfer.assets
    }
  })
})

const columns: Array<ColumnDef<{ chain_id: string }>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    size: 200,
    cell: info => flexRender(CellOrigin, { value: info.getValue() })
  },
  {
    accessorKey: "destination",
    header: () => "Destination",
    size: 200,
    cell: info => flexRender(CellOrigin, { value: info.getValue() })
  },
  {
    accessorKey: "sender",
    header: () => "Sender",
    size: 200,
    cell: info => truncate(info.getValue(), 8)
  },
  {
    accessorKey: "receiver",
    header: () => "Receiver",
    size: 200,
    cell: info => truncate(info.getValue(), 8)
  },
  {
    accessorKey: "assets",
    header: () => "Assets",
    size: 200,
    cell: info => flexRender(CellAssets, { value: info.getValue() })
  }
]
</script>

{#if !$cosmosStore?.rawAddress || !$sepoliaStore.address}
  <div>Connect your wallets to continue</div>

{:else}
  {#if $transfers.isLoading}
    <LoadingLogo class="size-16" />
  {:else if $transfers.isSuccess}
    <Table bind:dataStore={transfersData} {columns} />
  {/if}
{/if}
