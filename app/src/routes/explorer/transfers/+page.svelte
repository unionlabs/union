<script lang="ts">
import request from "graphql-request"
import { allTransfersQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived, writable } from "svelte/store"
import CellOrigin from "../(components)/cell-origin.svelte"
import CellAssets from "../(components)/cell-assets.svelte"
import CellDuration from "../(components)/cell-duration-text.svelte"
import { chainsQuery } from "$lib/queries/chains"
import { truncate } from "$lib/utilities/format"
import { goto } from "$app/navigation"
import LoadingLogo from "$lib/components/loading-logo.svelte"

let transfers = createQuery({
  queryKey: ["transfers"],
  refetchInterval: 3_000,
  queryFn: async () => (await request(URLS.GRAPHQL, allTransfersQueryDocument, {})).v0_transfers
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
      source: sourceDisplayName,
      destination: destinationDisplayName,
      sender: transfer.sender,
      receiver: transfer.receiver,
      assets: transfer.assets,
      timestamp: transfer.source_timestamp,
      source_transaction_hash: transfer.source_transaction_hash
    }
  })
})

const columns: Array<ColumnDef<{ chain_id: string }>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    size: 200,
    cell: info => info.getValue()
  },
  {
    accessorKey: "destination",
    header: () => "Destination",
    size: 200,
    cell: info => info.getValue()
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
  },
  {
    accessorKey: "timestamp",
    header: () => "Time",
    size: 200,
    cell: info => flexRender(CellDuration, { value: info.getValue() })
  }
]
</script>

{#if $transfers.isLoading}
  <LoadingLogo class="size-16"/>
{:else if $transfers.isSuccess}
  <Table bind:dataStore={transfersData} {columns} onClick={(x) => {
    // @ts-ignore
    goto(`/explorer/transfers/${x.source_transaction_hash}`)}
  }/>
{/if}
