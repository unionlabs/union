<script lang="ts">
import {
  flexRender,
  type ColumnDef,
  getCoreRowModel,
  type CellContext,
  type TableOptions,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import {
  latestTransfers,
  paginatedTransfers,
  type TransferAddress,
  decodeTimestampSearchParam,
  encodeTimestampSearchParam
} from "./paginated-transfers.ts"
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Table from "$lib/components/ui/table"
import { goto, onNavigate } from "$app/navigation"
import { showUnsupported } from "$lib/stores/user.ts"
import DevTools from "$lib/components/dev-tools.svelte"
import type { Chain, TransferAsset } from "$lib/types.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import CellAssets from "../(components)/cell-assets.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import { derived, writable, type Writable } from "svelte/store"
import CellOriginTransfer from "../(components)/cell-origin-transfer.svelte"
import { ExplorerPagination } from "../(components)/explorer-pagination/index.ts"
import { createQuery, useQueryClient, keepPreviousData } from "@tanstack/svelte-query"
import { toPrettyDateTimeFormat, currentUtcTimestampWithBuffer } from "$lib/utilities/date.ts"

/**
 * the timestamp is the source of truth, used as query key and url search param
 */

const QUERY_LIMIT = 30
const REFRESH_INTERVAL = 5_000 // 5 seconds

let timestamp = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : currentUtcTimestampWithBuffer()
)

let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })

const queryClient = useQueryClient()

/**
 * only happens when:
 *  1. it is the first query on initial page load with no timestamp search param,
 *  2. the user clicks on the `current` button which resets to current and live data
 */
let REFETCH_ENABLED = writable($page.url.searchParams.has("timestamp") ? false : true)

let liveTransfers = createQuery(
  derived(REFETCH_ENABLED, $REFETCH_ENABLED => ({
    queryKey: ["transfers", "live"],
    staleTime: Number.POSITIVE_INFINITY,
    enabled: $REFETCH_ENABLED,
    refetchOnMount: $REFETCH_ENABLED,
    placeholderData: keepPreviousData,
    refetchOnReconnect: $REFETCH_ENABLED,
    refetchInterval: () => ($REFETCH_ENABLED ? REFRESH_INTERVAL : false),
    queryFn: async () => await latestTransfers({ limit: QUERY_LIMIT * 2 })
  }))
)

let transfers = createQuery(
  derived([timestamp, REFETCH_ENABLED], ([$timestamp, $REFETCH_ENABLED]) => ({
    queryKey: ["transfers", $timestamp],
    staleTime: Number.POSITIVE_INFINITY,
    refetchOnMount: false,
    refetchOnReconnect: false,
    enabled: () => $REFETCH_ENABLED === false,
    placeholderData: keepPreviousData,
    queryFn: async () =>
      await paginatedTransfers({
        timestamp: $timestamp,
        limit: QUERY_LIMIT
      })
  }))
)

let queryStatus: "pending" | "done" = $REFETCH_ENABLED
  ? $liveTransfers.status === "pending" || $liveTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $transfers.status === "pending" || $transfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
$: queryStatus = $REFETCH_ENABLED
  ? $liveTransfers.status === "pending" || $liveTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $transfers.status === "pending" || $transfers.fetchStatus === "fetching"
    ? "pending"
    : "done"

let transfersDataStore = derived(
  [liveTransfers, transfers, REFETCH_ENABLED],
  ([$liveTransfers, $transfers, $REFETCH_ENABLED]) => {
    if ($REFETCH_ENABLED) return $liveTransfers?.data?.transfers ?? []
    return $transfers?.data?.transfers ?? []
  }
)

type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

let timestamps = derived(
  [liveTransfers, transfers, REFETCH_ENABLED],
  ([$liveTransfers, $transfers, $REFETCH_ENABLED]) =>
    $REFETCH_ENABLED
      ? {
          oldestTimestamp: $liveTransfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $liveTransfers?.data?.latestTimestamp ?? ""
        }
      : {
          oldestTimestamp: $transfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $transfers?.data?.latestTimestamp ?? ""
        }
)

const columns: Array<ColumnDef<DataRow, TransferAddress>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    size: 200,
    minSize: 200,
    maxSize: 200,
    accessorFn: (originalRow, _index) => originalRow.source,
    cell: _info => {
      const info = _info as CellContext<DataRow, TransferAddress> & {
        chains: Array<Chain>
      }
      const { chainId, address } = info.getValue()
      const chainDisplayName =
        info.chains.find(chain => chain.chain_id === chainId)?.display_name ?? "unknown chain"
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
    size: 200,
    minSize: 200,
    maxSize: 200,
    accessorFn: (originalRow, _index) => originalRow.destination,
    cell: _info => {
      const info = _info as CellContext<DataRow, TransferAddress> & {
        chains: Array<Chain>
      }
      const { chainId, address } = info.getValue()
      const chainDisplayName =
        info.chains.find(chain => chain.chain_id === chainId)?.display_name ?? "unknown chain"
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
    size: 200,
    minSize: 200,
    maxSize: 200,
    cell: info => flexRender(CellAssets, { value: info.getValue() })
  },
  {
    accessorKey: "timestamp",
    header: () => "Time",
    size: 200,
    minSize: 200,
    maxSize: 200,
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

$: if ($transfersDataStore) rerender()

/**
 * this can be removed if desired
 * it is only used to clear the cache when navigating away from the page `/explorer/transfers`
 */
onNavigate(navigation => {
  if (navigation.to?.route.id !== "/explorer/transfers") {
    queryClient.removeQueries({ queryKey: ["transfers"] })
  }
})
</script>

<DevTools>
  {JSON.stringify(
    { idx: $pagination.pageIndex, $REFETCH_ENABLED },
    undefined,
    2
  )}
</DevTools>
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
                  <ChainsGate let:chains>
                    <svelte:component
                      this={flexRender(cell.column.columnDef.cell, {
                        ...cell.getContext(),
                        chains
                      })}
                    />
                  </ChainsGate>
                </a>
              </Table.Cell>
            {/each}
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
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
    pageNumber={$pagination.pageIndex + 1}
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
      goto("/explorer/transfers", { replaceState: true })
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
