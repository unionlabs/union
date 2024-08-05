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
import {
  type TransferAddress,
  latestAddressesTransfers,
  decodeTimestampSearchParam,
  encodeTimestampSearchParam,
  paginatedAddressesTransfers
} from "./paginated-transfers.ts"
import { page } from "$app/stores"
import { getContext } from "svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Table from "$lib/components/ui/table"
import { goto, onNavigate } from "$app/navigation"
import { showUnsupported } from "$lib/stores/user.ts"
import DevTools from "$lib/components/dev-tools.svelte"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, TransferAsset } from "$lib/types.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
// import { addressTransfersPreference } from "../preference.ts"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellAssets from "../../(components)/cell-assets.svelte"
import { derived, writable, type Readable } from "svelte/store"
import CellTooltipIcon from "../../(components)/cell-icon-tooltip.svelte"
import CellOriginTransfer from "../../(components)/cell-origin-transfer.svelte"
import { ExplorerPagination } from "../../(components)/explorer-pagination/index.ts"
import { createQuery, useQueryClient, keepPreviousData } from "@tanstack/svelte-query"
import { toPrettyDateTimeFormat, currentUtcTimestampWithBuffer } from "$lib/utilities/date.ts"

// addressTransfersPreference.useLocalStorage()

let QUERY_LIMIT = 10
let REFRESH_INTERVAL = 5_000

let timestamp = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : currentUtcTimestampWithBuffer()
)

let pagination = writable({ pageIndex: 1, pageSize: QUERY_LIMIT })

const queryClient = useQueryClient()

let addressArray =
  getContext<Readable<{ nonNormalized: Array<string>; normalized: Array<string> }>>("addressArray")

let nonNormalizedAddressArray = $addressArray.nonNormalized

let normalizedAddressArray = derived(addressArray, $addressArray => $addressArray.normalized)
/**
 * only happens when:
 *  1. it is the first query on initial page load with no timestamp search param,
 *  2. the user clicks on the `current` button which resets to current and live data
 */
let REFETCH_ENABLED = writable($page.url.searchParams.has("timestamp") ? false : true)

let liveAddressTransfers = createQuery(
  derived(
    [REFETCH_ENABLED, normalizedAddressArray],
    ([$REFETCH_ENABLED, $normalizedAddressArray]) => ({
      queryKey: ["address-transfers", "live", ...$normalizedAddressArray],
      refetchOnMount: $REFETCH_ENABLED,
      placeholderData: keepPreviousData,
      staleTime: Number.POSITIVE_INFINITY,
      refetchOnReconnect: $REFETCH_ENABLED,
      enabled: $REFETCH_ENABLED,
      refetchInterval: () => ($REFETCH_ENABLED ? REFRESH_INTERVAL : false),
      queryFn: async () =>
        await latestAddressesTransfers({
          limit: QUERY_LIMIT * 2,
          addresses: $normalizedAddressArray
        })
    })
  )
)

let addressTransfers = createQuery(
  derived(
    [timestamp, normalizedAddressArray, REFETCH_ENABLED],
    ([$timestamp, $normalizedAddressArray, $REFETCH_ENABLED]) => ({
      queryKey: ["address-transfers", $timestamp, ...$normalizedAddressArray],
      refetchOnMount: false,
      refetchOnReconnect: false,
      placeholderData: keepPreviousData,
      staleTime: Number.POSITIVE_INFINITY,
      enabled: () => $REFETCH_ENABLED === false,
      queryFn: async () =>
        await paginatedAddressesTransfers({
          limit: QUERY_LIMIT,
          timestamp: $timestamp,
          addresses: $normalizedAddressArray
        })
    })
  )
)

let queryStatus: "pending" | "done" = $REFETCH_ENABLED
  ? $liveAddressTransfers.status === "pending" || $liveAddressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $addressTransfers.status === "pending" || $addressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
$: queryStatus = $REFETCH_ENABLED
  ? $liveAddressTransfers.status === "pending" || $liveAddressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $addressTransfers.status === "pending" || $addressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"

let transfersDataStore = derived(
  [liveAddressTransfers, addressTransfers, REFETCH_ENABLED],
  ([$liveAddressTransfers, $addressTransfers, $REFETCH_ENABLED]) => {
    if ($REFETCH_ENABLED) return $liveAddressTransfers?.data?.transfers ?? []
    return $addressTransfers?.data?.transfers ?? []
  }
)

type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

let timestamps = derived(
  [liveAddressTransfers, addressTransfers, REFETCH_ENABLED],
  ([$liveAddressTransfers, $addressTransfers, $REFETCH_ENABLED]) =>
    $REFETCH_ENABLED
      ? {
          oldestTimestamp: $liveAddressTransfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $liveAddressTransfers?.data?.latestTimestamp ?? ""
        }
      : {
          oldestTimestamp: $addressTransfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $addressTransfers?.data?.latestTimestamp ?? ""
        }
)

const columns: Array<ColumnDef<DataRow>> = [
  {
    size: 100,
    minSize: 20,
    maxSize: 100,
    accessorKey: "hash",
    header: _ => "Tx Hash",
    accessorFn: (originalRow, _index) => originalRow.hash,
    cell: info => {
      const destinationRecord =
        !info.row.original.destination.hash || info.row.original.destination.hash === "unknown"
          ? undefined
          : {
              truncateSize: 12,
              index: info.row.index,
              label: "Destination Tx",
              hash: info.row.original.destination.hash
            }
      return flexRender(CellTooltipIcon, {
        records: [
          {
            truncateSize: 14,
            label: "Source Tx",
            hash: info.getValue(),
            index: info.row.index
          },
          destinationRecord
        ].filter(Boolean)
      })
    }
  },
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

$: console.info("rows per page", $rows.length)

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
  if (!navigation.to?.route.id?.startsWith("/explorer/address")) {
    queryClient.removeQueries({ queryKey: ["address-transfers"] })
  }
})
</script>

<DevTools>
  <pre>
    {JSON.stringify(
      {
        idx: $pagination.pageIndex,
        $REFETCH_ENABLED,
        $timestamp,
        ...$timestamps,
        ...$addressArray
      },
      undefined,
      2
    )}
  </pre>
</DevTools>

{#if $transfersDataStore?.length}
  <Card.Root>
    <Table.Root>
      <ChainsGate let:chains>
        <Table.Header class="tabular-nums">
          {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
            <Table.Row class="tabular-nums">
              {#each headerGroup.headers as header, index (header.id)}
                <Table.Head
                  colspan={header.colSpan}
                  class={cn(
                    index === 0 ? "pl-5" : "",
                    `w-[${header.getSize()}px]`,
                    "whitespace-nowrap tabular-nums"
                  )}
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
                {@const columnId = cell.column.id}
                {@const hash = $rows[row.index].original.hash}
                <Table.Cell class={cn("tabular-nums h-12")} headers="header">
                  {#if columnId === "hash"}
                    <svelte:component
                      this={flexRender(cell.column.columnDef.cell, {
                        ...cell.getContext(),
                        chains
                      })}
                    />
                  {:else}
                    <a href={`/explorer/transfers/${hash}`}>
                      <svelte:component
                        this={flexRender(cell.column.columnDef.cell, {
                          ...cell.getContext(),
                          chains
                        })}
                      />
                    </a>
                  {/if}
                </Table.Cell>
              {/each}
            </Table.Row>
          {/each}
        </Table.Body>
      </ChainsGate>
    </Table.Root>
  </Card.Root>
{:else if queryStatus === "pending"}
  <LoadingLogo class="size-16" />
{/if}
<div
  class="flex sm:justify-start sm:flex-row flex-col justify-end items-end gap-1 w-full"
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
      pagination.update((p) => ({ ...p, pageIndex: 1 }))
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
