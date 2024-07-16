<script lang="ts">
  import {
    flexRender,
    type ColumnDef,
    getCoreRowModel,
    type TableOptions,
    createSvelteTable,
    getFilteredRowModel,
    getPaginationRowModel,
  } from '@tanstack/svelte-table'
  import {
    currentUtcTimestamp,
    toPrettyDateTimeFormat,
    currentUtcTimestampWithBuffer,
  } from '$lib/utilities/date.ts'
  import {
    latestTransfers,
    paginatedTransfers,
    decodeTimestampSearchParam,
    encodeTimestampSearchParam,
    transfersAfterOrAtTimestamp,
    transfersBeforeOrAtTimestamp,
  } from './paginated-transfers.ts'
  import { onDestroy } from 'svelte'
  import { page } from '$app/stores'
  import { cn } from '$lib/utilities/shadcn.ts'
  import { derived, writable, type Writable } from 'svelte/store'
  import * as Table from '$lib/components/ui/table'
  import { goto, onNavigate } from '$app/navigation'
  import { showUnsupported } from '$lib/stores/user.ts'
  import DevTools from '$lib/components/dev-tools.svelte'
  import * as Card from '$lib/components/ui/card/index.ts'
  import CellAssets from '../(components)/cell-assets.svelte'
  import LoadingLogo from '$lib/components/loading-logo.svelte'
  import type { UnwrapReadable } from '$lib/utilities/types.ts'
  import CellDuration from '../(components)/cell-duration-text.svelte'
  import CellOriginTransfer from '../(components)/cell-origin-transfer.svelte'
  import { ExplorerPagination } from '../(components)/explorer-pagination/index.ts'
  import { createQuery, keepPreviousData, useQueryClient } from '@tanstack/svelte-query'

  /**
   * the timestamp is the source of trust, used as query key and url search param
   */

  const QUERY_LIMIT = 8
  const REFRESH_INTERVAL = 5_000 // 5 seconds

  // minus 1 to account for the 0-based index
  let timestamp = writable(
    $page.url.searchParams.has('timestamp')
      ? decodeTimestampSearchParam(`${$page.url.searchParams.get('timestamp')}`)
      : currentUtcTimestampWithBuffer(),
  )
  // $: console.info($timestamp)

  let CURSOR: Writable<'ON_OR_BEFORE' | 'ON_OR_AFTER'> = writable('ON_OR_BEFORE')
  let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })

  // CURSOR.subscribe(value => {
  //   if (value === 'ON_OR_AFTER') {
  //     pagination.update(p => ({ ...p, pageIndex: 0 }))
  //   } else {
  //     pagination.update(p => ({ ...p, pageIndex: 0 }))
  //   }
  // })
  const queryClient = useQueryClient()

  /**
   * only happens when:
   *  1. it is the first query on initial page load,
   *  2. the user clicks on the `current` button which resets to current and live data
   */
  let REFETCH_ENABLED = writable($page.url.searchParams.has('timestamp') ? false : true)

  let liveTransfers = createQuery(
    derived(REFETCH_ENABLED, $REFETCH_ENABLED => ({
      queryKey: ['transfers', 'live'],
      // staleTime: () => ($REFETCH_ENABLED ? 0 : REFRESH_INTERVAL),
      staleTime: Number.POSITIVE_INFINITY,
      enabled: $REFETCH_ENABLED,
      refetchOnMount: $REFETCH_ENABLED,
      placeholderData: keepPreviousData,
      refetchOnReconnect: $REFETCH_ENABLED,
      refetchInterval: () => ($REFETCH_ENABLED ? REFRESH_INTERVAL : false),
      queryFn: async () => await latestTransfers({ limit: QUERY_LIMIT }),
    })),
  )

  $: console.info({ $CURSOR })

  let transfers = createQuery(
    derived([timestamp, REFETCH_ENABLED, CURSOR], ([$timestamp, $REFETCH_ENABLED, $CURSOR]) => ({
      queryKey: ['transfers', $timestamp],
      staleTime: Number.POSITIVE_INFINITY,
      // staleTime: ($REFETCH_ENABLED ? REFRESH_INTERVAL : 0),
      refetchOnMount: false,
      refetchOnReconnect: false,
      enabled: () => $REFETCH_ENABLED === false,
      placeholderData: keepPreviousData,
      // refetchInterval: () => ($REFETCH_ENABLED ? REFRESH_INTERVAL : false),
      queryFn: async () => {
        // console.info($CURSOR)
        return $CURSOR === 'ON_OR_AFTER'
          ? await transfersAfterOrAtTimestamp({
              timestamp: $timestamp,
              limit: QUERY_LIMIT,
            })
          : await transfersBeforeOrAtTimestamp({
              timestamp: $timestamp,
              limit: QUERY_LIMIT,
            })
      },
    })),
  )

  // $: console.info(JSON.stringify($transfers.data, undefined, 2))

  let queryStatus: 'pending' | 'done' = $REFETCH_ENABLED
    ? $liveTransfers.status === 'pending' || $liveTransfers.fetchStatus === 'fetching'
      ? 'pending'
      : 'done'
    : $transfers.status === 'pending' || $transfers.fetchStatus === 'fetching'
      ? 'pending'
      : 'done'
  $: queryStatus = $REFETCH_ENABLED
    ? $liveTransfers.status === 'pending' || $liveTransfers.fetchStatus === 'fetching'
      ? 'pending'
      : 'done'
    : $transfers.status === 'pending' || $transfers.fetchStatus === 'fetching'
      ? 'pending'
      : 'done'

  let transfersDataStore = derived(
    [liveTransfers, transfers, REFETCH_ENABLED],
    ([$liveTransfers, $transfers, $REFETCH_ENABLED]) => {
      if ($REFETCH_ENABLED) return $liveTransfers?.data?.transfers ?? []
      return $transfers?.data?.transfers ?? []
    },
  )

  // $: console.info($transfersDataStore)

  // $: hasNewer = $transfers?.data?.hasNewer
  // $: hasOlder = $transfers?.data?.hasOlder

  // $: if (!hasNewer) {
  //   $REFETCH_ENABLED = true
  //   pagination.update(p => ({ ...p, pageIndex: 0 }))
  // }

  type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

  let timestamps = derived(
    [liveTransfers, transfers, REFETCH_ENABLED],
    ([$liveTransfers, $transfers, $REFETCH_ENABLED]) =>
      $REFETCH_ENABLED
        ? {
            oldestTimestamp: $liveTransfers?.data?.oldestTimestamp ?? '',
            latestTimestamp: $liveTransfers?.data?.latestTimestamp ?? '',
          }
        : {
            oldestTimestamp: $transfers?.data?.oldestTimestamp ?? '',
            latestTimestamp: $transfers?.data?.latestTimestamp ?? '',
          },
  )

  // const unsubscribeTimestamps = timestamps.subscribe(value => {
  //   if ($REFETCH_ENABLED) return
  //   if (value.latestTimestamp) {
  //     goto(encodeTimestampSearchParam(value.latestTimestamp), {
  //       noScroll: true,
  //       keepFocus: true,
  //       replaceState: true,
  //     })
  //   }
  // })

  // const unsubscribeTimestamp = timestamp.subscribe(value => {
  //   if($page.url.searchParams.has('timestamp')) return
  //   goto(encodeTimestampSearchParam(value))
  // })

  const columns: Array<ColumnDef<DataRow>> = [
    {
      accessorKey: 'source',
      header: () => 'Source',
      size: 200,
      minSize: 200,
      maxSize: 200,
      cell: info => flexRender(CellOriginTransfer, { value: info.getValue() }),
    },
    {
      accessorKey: 'destination',
      header: () => 'Destination',
      size: 200,
      minSize: 200,
      maxSize: 200,
      cell: info => flexRender(CellOriginTransfer, { value: info.getValue() }),
    },
    {
      accessorKey: 'assets',
      header: () => 'Assets',
      size: 200,
      minSize: 200,
      maxSize: 200,
      cell: info => flexRender(CellAssets, { value: info.getValue() }),
    },
    {
      accessorKey: 'timestamp',
      header: () => 'Time',
      size: 200,
      minSize: 200,
      maxSize: 200,
      // @ts-ignore
      cell: info => toPrettyDateTimeFormat(info.getValue(), { local: true }),
    },
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
      pagination: $pagination,
    },
    // debugTable: import.meta.env.MODE === 'development',
  })

  const rerender = () => {
    options.update(options => ({
      ...options,
      data: $transfersDataStore,
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
    return Object.hasOwn(assets[info], 'info')
  }

  $: if ($transfersDataStore) rerender()

  onDestroy(() => {
    // unsubscribeTimestamp()
    // unsubscribeTimestamps()
  })

  /**
   * this can be removed if desired
   * it is only used to clear the cache when navigating away from the page `/explorer/transfers`
   */
  onNavigate(navigation => {
    if (navigation.to?.route.id !== '/explorer/transfers') {
      queryClient.removeQueries({ queryKey: ['transfers'] })
    }
  })
</script>

<DevTools>
  <pre>{JSON.stringify(
      {
        idx: $pagination.pageIndex,
        $REFETCH_ENABLED,
        firstHash: $transfersDataStore?.[0]?.hash,
        firstTimestamp: $transfersDataStore?.[0]?.timestamp,
        lastHash: $transfersDataStore?.[$transfersDataStore?.length - 1]?.hash,
        lastTimestamp: $transfersDataStore?.[$transfersDataStore?.length - 1]?.timestamp,
      },
      undefined,
      2,
    )}</pre>
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
                    this={flexRender(cell.column.columnDef.cell, cell.getContext())}
                  />
                </a>
              </Table.Cell>
            {/each}
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </Card.Root>
{:else if queryStatus === 'pending'}
  <LoadingLogo class="size-16" />
{/if}
<div class="flex sm:justify-start sm:flex-row flex-col justify-center gap-1">
  <ExplorerPagination
    class={cn('w-min')}
    rowsPerPage={20}
    totalTableRows={20}
    status={queryStatus}
    onOlderPage={page => {
      timestamp.set($timestamps.oldestTimestamp)
      goto(encodeTimestampSearchParam($timestamps.oldestTimestamp), { replaceState: true })
      pagination.update(p => ({ ...p, pageIndex: p.pageIndex + 1 }))
      $REFETCH_ENABLED = false
      // CURSOR = 'ON_OR_BEFORE'
      // CURSOR.set('ON_OR_BEFORE')
      CURSOR.update(c => 'ON_OR_BEFORE')
    }}
    onCurrentClick={() => {
      // timestamp.set(currentUtcTimestamp())
      pagination.update(p => ({ ...p, pageIndex: 0 }))
      $REFETCH_ENABLED = true
      goto('/explorer/transfers', { replaceState: true })
    }}
    onNewerPage={() => {
      timestamp.set($timestamps.latestTimestamp)
      console.info($timestamp, $timestamps.latestTimestamp)
      goto(encodeTimestampSearchParam($timestamps.latestTimestamp), { replaceState: true })
      pagination.update(p => ({ ...p, pageIndex: p.pageIndex - 1 }))
      $REFETCH_ENABLED = false
      // CURSOR = 'ON_OR_AFTER'
      // CURSOR.set('ON_OR_AFTER')
      CURSOR.update(c => 'ON_OR_AFTER')
    }}
  />
  <time class="font-normal text-md uppercase font-mono w-full my-auto sm:text-left text-center">
    {$timestamps.latestTimestamp ? toPrettyDateTimeFormat($timestamps.latestTimestamp) : ''}
  </time>
</div>
