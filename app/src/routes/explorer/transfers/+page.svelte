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
import { raise } from "$lib/utilities"
import { onMount } from "svelte"

let transfers = createQuery({
  queryKey: ["transfers-all"],
  placeholderData: (previousData, _) => previousData,
  retryDelay: attempt => Math.min(attempt > 1 ? 2 ** attempt * 1000 : 1000, 30 * 1000), // expo backoff
  refetchInterval: 5_000,
  queryFn: async () => {
    const response = await request(URLS.GRAPHQL, allTransfersQueryDocument, {})
    if (response.v0_transfers === undefined || response.v0_transfers === null)
      raise("error fetching transfers")
    return response.v0_transfers
  }
})

let chains = chainsQuery()

let processedTransfers = derived([transfers, chains], ([$transfers, $chains]) => {
  if (!$transfers.data || !$chains.data) return undefined
  return $transfers.data.map(tx => {
    let destinationChainId = tx.destination_chain_id
    let receiver = tx.receiver

    // overwrite destination and receiver if to last forward
    const lastForward = tx.forwards?.at(-1)
    if (lastForward && lastForward.receiver !== null && lastForward.chain !== null) {
      receiver = lastForward.receiver
      destinationChainId = lastForward.chain.chain_id
    }

    const sourceDisplayName = $chains.data.find(
      chain => chain.chain_id === tx.source_chain_id
    )?.display_name
    const destinationDisplayName = $chains.data.find(
      chain => chain.chain_id === destinationChainId
    )?.display_name

    return {
      ...tx,
      source: sourceDisplayName,
      destination: destinationDisplayName,
      timestamp: tx.source_timestamp,
      receiver
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

{#if !!$transfers.data && !!$processedTransfers}
  <Table bind:dataStore={processedTransfers} {columns} onClick={(x) => {
    // @ts-ignore
    goto(`/explorer/transfers/${x.source_transaction_hash}`)
  }}/>
{:else if $transfers.isLoading}
  <LoadingLogo class="size-16"/>
{:else if $transfers.isError}
  Error fetching transfers...
{/if}
