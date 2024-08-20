<script lang="ts">
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import Table from "../../../routes/explorer/(components)/table.svelte"
import type { Chain } from "$lib/types.ts"
import type { Transfer } from "./transfers-types.ts"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellAssets from "$lib/components/table-cells/cell-assets.svelte"
import CellTimestamp from "$lib/components/table-cells/cell-timestamp.svelte"
import { derived, type Readable } from "svelte/store"
import CellOriginTransfer from "$lib/components/table-cells/cell-origin-transfer.svelte"
import ExplorerPagination from "./explorer-pagination.svelte"
import { transfersQuery } from "$lib/queries/transfers.ts"
import { timestamp } from "$lib/stores/page.ts"

export let chains: Array<Chain>
export let normalizedAddresses: Array<string> | null = null
export let pageSize: number // must be even

const transfers = transfersQuery(normalizedAddresses, timestamp, pageSize)

const transfersDataStore: Readable<Array<Transfer>> = derived([transfers], ([$transfers]) => {
  return $transfers?.data?.map(d => ({ url: `/explorer/transfers/${d.hash}`, ...d })) ?? []
})

type DataRow = UnwrapReadable<typeof transfersDataStore>[number]
const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "source",
    cell: info => flexRender(CellOriginTransfer, { chains, value: info.getValue() })
  },
  {
    accessorKey: "destination",
    cell: info => flexRender(CellOriginTransfer, { chains, value: info.getValue() })
  },
  {
    accessorKey: "assets",
    cell: info => flexRender(CellAssets, { value: info.getValue() })
  },
  {
    header: () => "Time",
    accessorKey: "timestamp",
    cell: info => flexRender(CellTimestamp, { value: info.getValue() })
  }
]
</script>

{#if $transfers.data}
  <Table dataStore={transfersDataStore} {columns} />
  <ExplorerPagination explorerItems={transfersDataStore} />
{:else if $transfers.status  === "pending"}
  <LoadingLogo class="size-16" />
{/if}

<style lang="postcss"></style>
