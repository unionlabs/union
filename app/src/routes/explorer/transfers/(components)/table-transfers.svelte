<script lang="ts">
import {
  flexRender,
  type ColumnDef,
  getCoreRowModel,
  type CellContext,
  createSvelteTable,
  type TableOptions,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import { page } from "$app/stores"
import { goto } from "$app/navigation"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Table from "$lib/components/ui/table"
import { showUnsupported } from "$lib/stores/user.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, TransferAsset } from "$lib/types.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"
import type { Transfer, TransferAddress } from "../types.ts"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellAssets from "../../(components)/cell-assets.svelte"
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"
import { derived, writable, type Readable, type Writable } from "svelte/store"
import CellOriginTransfer from "../../(components)/cell-origin-transfer.svelte"
import { ExplorerPagination } from "../../(components)/explorer-pagination/index.ts"

export let transfersDataStore: Readable<Array<Transfer>>

type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

export let queryStatus: "pending" | "done"
export let REFETCH_ENABLED: Writable<boolean>
export let timestamp: Writable<string>
export let timestamps: Readable<{
  oldestTimestamp: string
  latestTimestamp: string
}>
export let pagination: Writable<{ pageIndex: number; pageSize: number }>

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
    // @ts-expect-error
    cell: info => toPrettyDateTimeFormat(info.getValue(), { local: true })
  }
]

const options = writable<TableOptions<DataRow>>({
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
  state: { pagination: $pagination },
  debugTable: import.meta.env.MODE === "development" && import.meta.env.DEBUG_TABLE === "true"
})

const rerender = () => {
  options.update(options => ({
    ...options,
    data: $transfersDataStore
  }))
}

const table = createSvelteTable(options)
const rows = derived(table, $t => $t.getRowModel().rows)

function assetHasInfoProperty(assets: TransferAsset) {
  const [[_, { info }]] = Object.entries(assets)
  return !!info
}

const encodeTimestampSearchParam = (timestamp: string) =>
  `?timestamp=${toPrettyDateTimeFormat(timestamp)?.replaceAll("-", "").replaceAll(":", "").replaceAll(" ", "")}`

$: if ($transfersDataStore) rerender()
</script>

{#if $transfersDataStore?.length}
  <Card.Root>
    <ChainsGate let:chains>
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
          {#each $table.getRowModel().rows as row, index (row.index)}
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
    </ChainsGate>
  </Card.Root>
{:else if queryStatus === "pending"}
  <LoadingLogo class="size-16" />
{/if}
<div
  class="flex sm:justify-start sm:flex-row flex-col justify-center gap-1 w-full"
>
  <ExplorerPagination
    rowsPerPage={20}
    totalTableRows={20}
    class={cn("w-auto")}
    status={queryStatus}
    live={$REFETCH_ENABLED}
    onOlderPage={async (page) => {
      const stamp = $timestamps.oldestTimestamp
      timestamp.set(stamp)
      goto(encodeTimestampSearchParam(stamp), {
        replaceState: true,
        state: { timestamp: stamp }
      })
      pagination.update((p) => ({ ...p, pageIndex: p.pageIndex + 1 }))
      $REFETCH_ENABLED = false
    }}
    onCurrentClick={() => {
      pagination.update((p) => ({ ...p, pageIndex: 0 }))
      $REFETCH_ENABLED = true
      goto($page.url.pathname, { replaceState: true })
    }}
    onNewerPage={async (page) => {
      const stamp = $timestamps.latestTimestamp
      timestamp.set(stamp)
      goto(encodeTimestampSearchParam(stamp), {
        replaceState: true,
        state: { timestamp: stamp }
      })
      pagination.update((p) => ({ ...p, pageIndex: p.pageIndex - 1 }))
      $REFETCH_ENABLED = false
    }}
    timestamp={$timestamps.latestTimestamp
      ? toPrettyDateTimeFormat($timestamps.latestTimestamp, { local: true })
      : ""}
  />
</div>

<style lang="postcss"></style>
