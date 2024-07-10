<script lang="ts">
import {
  flexRender,
  type ColumnDef,
  getCoreRowModel,
  type TableOptions,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import {
  currentUtcTimestamp,
  toPrettyDateTimeFormat,
  currentUtcTimestampWithBuffer
} from "$lib/utilities/date.ts"
import {
  paginatedTransfers,
  decodeTimestampSearchParam,
  encodeTimestampSearchParam
} from "./paginated-transfers.ts"
import { onDestroy } from "svelte"
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import { derived, writable } from "svelte/store"
import * as Table from "$lib/components/ui/table"
import { goto, onNavigate } from "$app/navigation"
import { showUnsupported } from "$lib/stores/user.ts"
import DevTools from "$lib/components/dev-tools.svelte"
import * as Card from "$lib/components/ui/card/index.ts"
import CellAssets from "../(components)/cell-assets.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import type { UnwrapReadable } from "$lib/utilities/types.ts"
import CellDuration from "../(components)/cell-duration-text.svelte"
import CellOriginTransfer from "../(components)/cell-origin-transfer.svelte"
import { ExplorerPagination } from "../(components)/explorer-pagination/index.ts"
import { createQuery, keepPreviousData, useQueryClient } from "@tanstack/svelte-query"

/**
 * the timestamp is the source of trust, used as query key and url search param
 */

const QUERY_LIMIT = 12 // 12 x 2 = 24
const REFRESH_INTERVAL = 5_000 // 5 seconds

// minus 1 to account for the 0-based index
let timestamp = writable(currentUtcTimestamp())
$: console.info($timestamp)
let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })

const queryClient = useQueryClient()

/**
 * only happens when:
 *  1. it is the first query on initial page load,
 *  2. the user clicks on the `current` button which resets to current and live data
 */
let REFETCH_ENABLED = true

let transfers = createQuery(
  derived([timestamp, pagination], ([$timestamp, $pagination]) => ({
    queryKey: ["transfers", $timestamp],
    staleTime: REFRESH_INTERVAL,
    refetchOnMount: REFETCH_ENABLED,
    placeholderData: keepPreviousData,
    refetchOnReconnect: REFETCH_ENABLED,
    refetchInterval: () => (REFETCH_ENABLED ? REFRESH_INTERVAL : false),
    queryFn: async () => await paginatedTransfers({ limit: QUERY_LIMIT, timestamp: $timestamp })
  }))
)

let queryStatus: "pending" | "done" =
  $transfers.status === "pending" || $transfers.fetchStatus === "fetching" ? "pending" : "done"
$: queryStatus =
  $transfers.status === "pending" || $transfers.fetchStatus === "fetching" ? "pending" : "done"

let transfersDataStore = derived(transfers, $transfers => $transfers?.data?.transfers ?? [])

$: hasNewer = $transfers?.data?.hasNewer
$: hasOlder = $transfers?.data?.hasOlder

$: if (!hasNewer) {
  REFETCH_ENABLED = true
  pagination.update(p => ({ ...p, pageIndex: 0 }))
}

type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

let timestamps = derived(transfers, $transfers => ({
  oldestTimestamp: $transfers?.data?.oldestTimestamp ?? "",
  latestTimestamp: $transfers?.data?.latestTimestamp ?? ""
}))

const unsubscribeTimestamps = timestamps.subscribe(value => {
  if (REFETCH_ENABLED && value.latestTimestamp) {
    goto(encodeTimestampSearchParam(value.latestTimestamp), {
      noScroll: true,
      keepFocus: true,
      replaceState: true
    })
  }
})

const unsubscribeTimestamp = timestamp.subscribe(value => {
  goto(encodeTimestampSearchParam(value))
})

const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "source",
    header: () => "Source",
    size: 200,
    minSize: 200,
    maxSize: 200,
    cell: info => flexRender(CellOriginTransfer, { value: info.getValue() })
  },
  {
    accessorKey: "destination",
    header: () => "Destination",
    size: 200,
    minSize: 200,
    maxSize: 200,
    cell: info => flexRender(CellOriginTransfer, { value: info.getValue() })
  },
  {
    accessorKey: "assets",
    header: () => "Assets",
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
    // @ts-ignore
    cell: info => toPrettyDateTimeFormat(info.getValue())
  }
]

const options = writable<TableOptions<DataRow>>({
  data: $transfersDataStore,
  columns,
  enableHiding: true,
  enableFilters: true,
  rowCount: $transfersDataStore?.length,
  autoResetPageIndex: true,
  enableColumnFilters: true,
  enableColumnResizing: true,
  enableMultiRowSelection: true,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  manualPagination: true,
  getPaginationRowModel: getPaginationRowModel(),
  state: {
    pagination: $pagination
  },
  debugTable: import.meta.env.MODE === "development"
})

const rerender = () => {
  options.update(options => ({
    ...options,
    data: $transfersDataStore
  }))
}

const table = createSvelteTable(options)
const rows = derived(table, $t => $t.getRowModel().rows)

function hasInfoProperty(assets: Object) {
  const info = Object?.keys(assets).at(0)
  if (!info) return false
  // @ts-ignore
  if (!assets[info]) return false
  // @ts-ignore
  return Object.hasOwn(assets[info], "info")
}

$: if ($transfersDataStore) rerender()

onDestroy(() => {
  unsubscribeTimestamp()
  unsubscribeTimestamps()
})

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

<DevTools />
{`${JSON.stringify({ REFETCH_ENABLED, idx: $pagination.pageIndex }, undefined, 2)}`}
{#if $transfers?.data}
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
                  this={flexRender(header.column.columnDef.header, header.getContext())}
                />
              </Table.Head>
            {/each}
          </Table.Row>
        {/each}
      </Table.Header>
      <Table.Body class={cn(`whitespace-nowrap h-full tabular-nums`)}>
        {#each $table.getRowModel().rows as row, index (row.index)}
          {@const isSupported = hasInfoProperty($rows[row.index]?.original?.assets)}
          {@const showUnsupported = $showUnsupported}
          <Table.Row
            class={cn(
              'cursor-pointer tabular-nums',
              index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
              isSupported ? '' : 'opacity-50',
            )}
            on:click={() => goto(`/explorer/transfers/${$rows[row.index].original.hash}`)}
          >
            {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
              <Table.Cell class="tabular-nums">
                <svelte:component
                  this={flexRender(cell.column.columnDef.cell, cell.getContext())}
                />
              </Table.Cell>
            {/each}
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </Card.Root>
{:else if $transfers.isLoading}
  <LoadingLogo class="size-16" />
{/if}
<div class="flex sm:justify-start sm:flex-row flex-col justify-center gap-1">
  <ExplorerPagination
    class={cn('w-min')}
    totalTableRows={20}
    status={queryStatus}
    currentPage={$pagination.pageIndex}
    bind:rowsPerPage={$pagination.pageSize}
    onOlderPage={page => {
      timestamp.set($timestamps.oldestTimestamp)
      pagination.update(p => ({ ...p, pageIndex: p.pageIndex + 1 }))
      REFETCH_ENABLED = false
    }}
    onCurrentClick={() => {
      timestamp.set(currentUtcTimestamp())
      pagination.update(p => ({ ...p, pageIndex: 0 }))
      REFETCH_ENABLED = true
    }}
    newerDisabled={!hasNewer}
    onNewerPage={() => {
      timestamp.set($timestamps.latestTimestamp)
      pagination.update(p => ({ ...p, pageIndex: p.pageIndex - 1 }))
      REFETCH_ENABLED = false
    }}
  />
  <time class="font-normal text-md uppercase font-mono w-full my-auto sm:text-left text-center">
    {$page.url.searchParams.get('timestamp')
      ? decodeTimestampSearchParam(`${$page.url.searchParams.get('timestamp')}`)
      : $timestamp}
  </time>
</div>
