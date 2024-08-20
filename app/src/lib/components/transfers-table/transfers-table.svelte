<script lang="ts">
import {
  flexRender,
  type ColumnDef,
  getCoreRowModel,
  type CellContext,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Table from "$lib/components/ui/table"
import { showUnsupported } from "$lib/stores/user.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, TransferAsset } from "$lib/types.ts"
import type { Transfer, TransferAddress } from "./transfers-types.ts"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellAssets from "$lib/components/table-cells/cell-assets.svelte"
import CellTimestamp from "$lib/components/table-cells/cell-timestamp.svelte"
import { derived, type Readable } from "svelte/store"
import CellOriginTransfer from "$lib/components/table-cells/cell-origin-transfer.svelte"
import ExplorerPagination from "./explorer-pagination.svelte"
import { createQuery, keepPreviousData } from "@tanstack/svelte-query"
import {
  transfersLive,
  transfersByTimestamp,
  transfersLiveByAddress,
  transfersByTimestampForAddresses,
  transfersQuery
} from "$lib/queries/transfers.ts"
import { timestamp } from "$lib/stores/page.ts"

export let chains: Array<Chain>
export let normalizedAddresses: Array<string> | null = null
export let pageSize: number // must be even

const transfers = transfersQuery(normalizedAddresses, timestamp, pageSize)

const transfersDataStore: Readable<Array<Transfer>> = derived([transfers], ([$transfers]) => {
  return $transfers?.data ?? []
})

type DataRow = UnwrapReadable<typeof transfersDataStore>[number]
const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    accessorFn: (originalRow, _index) => originalRow.source,
    cell: _info => {
      const info = _info as CellContext<DataRow, TransferAddress> & {
        chains: Array<Chain>
      }
      const { chainId, address } = info.getValue()
      const chainDisplayName =
        info.chains.find(chain => chain.chain_id === chainId)?.display_name ??
        chainId ??
        "unknown chain"
      return flexRender(CellOriginTransfer, {
        value: {
          address,
          chain_display_name: chainDisplayName
        }
      })
    }
  },
  {
    accessorKey: "destination",
    header: () => "Destination",
    accessorFn: (originalRow, _index) => originalRow.destination,
    cell: _info => {
      const info = _info as CellContext<DataRow, TransferAddress> & {
        chains: Array<Chain>
      }
      const { chainId, address } = info.getValue()
      const chainDisplayName =
        info.chains.find(chain => chain.chain_id === chainId)?.display_name ??
        chainId ??
        "unknown chain"
      return flexRender(CellOriginTransfer, {
        value: {
          address,
          chain_display_name: chainDisplayName
        }
      })
    }
  },
  {
    accessorKey: "assets",
    header: () => "Asset",
    cell: info => flexRender(CellAssets, { value: info.getValue() })
  },
  {
    header: () => "Time",
    accessorKey: "timestamp",
    cell: info => flexRender(CellTimestamp, { value: info.getValue() })
  }
]

const table = createSvelteTable(
  derived([transfersDataStore], ([$transfersDataStore]) => ({
    data: $transfersDataStore,
    columns,
    enableHiding: true,
    enableFilters: true,
    manualPagination: true,
    autoResetPageIndex: true,
    enableColumnFilters: true,
    enableColumnResizing: true,
    enableMultiRowSelection: true,
    getCoreRowModel: getCoreRowModel(),
    rowCount: $transfersDataStore?.length,
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    debugTable: import.meta.env.MODE === "development" && import.meta.env.DEBUG_TABLE === "true"
  }))
)
const rows = derived(table, $t => $t.getRowModel().rows)

function assetHasInfoProperty(assets: TransferAsset) {
  const [[_, { info }]] = Object.entries(assets)
  return !!info
}
</script>

{#if $transfersDataStore?.length}
  <Card.Root>
    <Table.Root>
      <Table.Header class="tabular-nums">
        {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <Table.Row class="tabular-nums">
            {#each headerGroup.headers as header (header.id)}
              <Table.Head
                colspan={header.colSpan}
                rowspan={header.rowSpan}
                class={cn(`whitespace-nowrap tabular-nums`)}
              >
                <svelte:component
                  this={flexRender(
                    header.column.columnDef.header,
                    header.getContext()
                  )}
                />
              </Table.Head>
            {/each}
          </Table.Row>
        {/each}
      </Table.Header>
      <Table.Body class={cn(`whitespace-nowrap h-full tabular-nums`)}>
        {#each $rows as row, index (row.index)}
          {@const isSupported = assetHasInfoProperty(
            $rows[row.index]?.original?.assets
          )}
          {@const showUnsupported = $showUnsupported}
          {@const shouldShow = isSupported || showUnsupported}
          <Table.Row
            class={cn(
              "cursor-pointer tabular-nums",
              index % 2 === 0 ? "bg-secondary/10" : "bg-transparent",
              isSupported ? "" : "opacity-50",
              shouldShow ? "" : "hidden"
            )}
          >
            {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
              {@const hash = $rows[row.index].original.hash}
              <Table.Cell class="tabular-nums" headers="header">
                <a
                  title={hash}
                  href={`/explorer/transfers/${hash}`}
                  class="size-full min-size-full w-full"
                >
                  <svelte:component
                    this={flexRender(cell.column.columnDef.cell, {
                      ...cell.getContext(),
                      chains
                    })}
                  />
                </a>
              </Table.Cell>
            {/each}
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </Card.Root>
  <ExplorerPagination explorerItems={transfersDataStore} />
{:else if $transfers.status  === "pending"}
  <LoadingLogo class="size-16" />
{/if}

<style lang="postcss"></style>
