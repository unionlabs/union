<script lang="ts">
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { goto } from "$app/navigation"
import Table from "../(components)/table.svelte"
import { derived, writable } from "svelte/store"
import { truncate } from "$lib/utilities/format"
import { chainsQuery } from "$lib/queries/chains"
import { createQuery } from "@tanstack/svelte-query"
import { rankItem } from "@tanstack/match-sorter-utils"
import CellAssets from "../(components)/cell-assets.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import CellPlainText from "../(components)/cell-plain-text.svelte"
import CellDuration from "../(components)/cell-duration-text.svelte"
import { allTransfersQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { flexRender, type ColumnDef, type FilterFn } from "@tanstack/svelte-table"

let transfers = createQuery({
  queryKey: ["transfers"],
  refetchInterval: 3_000,
  queryFn: async () => (await request(URLS.GRAPHQL, allTransfersQueryDocument, {})).v0_transfers
})

let chains = chainsQuery()

let transfersData = derived([transfers, chains], ([$transfers, $chains]) => {
  if (!($transfers.isSuccess && $chains.isSuccess)) return []
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

type DataRow = (typeof $transfersData)[number]

let globalFilter = ""
const fuzzyFilter = ((row, columnId, value, addMeta) => {
  const itemRank = rankItem(row.getValue(columnId), value)
  addMeta({ itemRank })
  return itemRank.passed
}) satisfies FilterFn<DataRow>

const columns: Array<ColumnDef<DataRow>> = [
  {
    size: 200,
    accessorKey: "source",
    header: () => "Source",
    filterFn: "includesString",
    accessorFn: row => row.source,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "destination",
    header: () => "Destination",
    filterFn: "includesString",
    accessorFn: row => row.destination,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "sender",
    header: () => "Sender",
    filterFn: "includesString",
    accessorFn: row => row.sender,
    cell: info => truncate(String(info.getValue()), 8)
  },
  {
    size: 200,
    accessorKey: "receiver",
    header: () => "Receiver",
    filterFn: "includesString",
    accessorFn: row => row.receiver,
    cell: info => truncate(String(info.getValue()), 8)
  },
  {
    size: 0,
    id: "hidden",
    header: () => "",
    enableHiding: true,
    filterFn: "includesString",
    accessorKey: "source_transaction_hash",
    accessorFn: row => row.source_transaction_hash,
    cell: info =>
      flexRender(CellPlainText, {
        value: info.getValue(),
        class: "hidden invisible size-0"
      })
  },
  {
    size: 200,
    accessorKey: "assets",
    header: () => "Assets",
    filterFn: "includesString",
    accessorFn: row => row.assets,
    cell: info => flexRender(CellAssets, { value: info.getValue() })
  },
  {
    size: 200,
    accessorKey: "timestamp",
    header: () => "Time",
    filterFn: "includesString",
    accessorFn: row => row.timestamp,
    cell: info => flexRender(CellDuration, { value: info.getValue() })
  }
]
</script>

{#if $transfers.isLoading}
  <LoadingLogo class="size-16" />
{:else if $transfers.isSuccess}
  <Table
    {columns}
    {fuzzyFilter}
    {globalFilter}
    tableName="Transfers"
    enableFiltering={true}
    bind:dataStore={transfersData}
    onClick={x => goto(`/explorer/transfers/${x.source_transaction_hash}`)}
  />
{/if}
