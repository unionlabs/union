<script lang="ts">
import request from "graphql-request"
import { allTransfersQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import Table from "../(components)/table.svelte"
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import CellAssets from "../(components)/cell-assets.svelte"
import CellDuration from "../(components)/cell-duration-text.svelte"
import CellOriginTransfer from "../(components)/cell-origin-transfer.svelte"
import { goto } from "$app/navigation"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { toDisplayName } from "$lib/utilities/chains.ts"
import type { Chain } from "$lib/types"
import type { UnwrapReadable } from "$lib/utilities/types"
import { raise } from "$lib/utilities"

export let chains: Array<Chain>

let transfers = createQuery({
  queryKey: ["transfers-all"],
  refetchInterval: 5_000,
  queryFn: async () => request(URLS.GRAPHQL, allTransfersQueryDocument, {}),
  select: data => {
    if (!data.v0_transfers) raise("error fetching transfers")

    return data.v0_transfers.map(tx => {
      let destinationChainId = tx.destination_chain_id
      let receiver = tx.receiver

      const lastForward = tx.forwards_2?.at(-1)
      if (lastForward && lastForward.receiver !== null && lastForward.chain !== null) {
        receiver = lastForward.receiver
        destinationChainId = lastForward.chain.chain_id
      }

      return {
        source: {
          chain_display_name: toDisplayName(tx.source_chain_id, chains),
          address: tx.sender || "unknown"
        },
        destination: {
          chain_display_name: toDisplayName(tx.destination_chain_id, chains),
          address: tx.receiver || "unknown"
        },
        assets: tx.assets,
        timestamp: tx.source_timestamp,
        source_transaction_hash: tx.source_transaction_hash
      }
    })
  }
})

let transfersDataStore = derived(transfers, $transfers => $transfers.data ?? [])

type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    size: 200,
    cell: info => flexRender(CellOriginTransfer, { value: info.getValue() })
  },
  {
    accessorKey: "destination",
    header: () => "Destination",
    size: 200,
    cell: info => flexRender(CellOriginTransfer, { value: info.getValue() })
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

{#if $transfers.data}
  <Table
    bind:dataStore={transfersDataStore}
    {columns}
    onClick={x => {
      goto(`/explorer/transfers/${x.source_transaction_hash}`)
    }}
  />
{:else if $transfers.isLoading}
  <LoadingLogo class="size-16" />
{:else if $transfers.isError}
  Error fetching transfers...
{/if}
