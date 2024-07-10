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
  import { onDestroy } from 'svelte'
  import { cn } from '$lib/utilities/shadcn.ts'
  import * as Table from '$lib/components/ui/table'
  import { showUnsupported } from '$lib/stores/user.ts'
  import DevTools from '$lib/components/dev-tools.svelte'
  import * as Card from '$lib/components/ui/card/index.ts'
  import CellAssets from '../(components)/cell-assets.svelte'
  import LoadingLogo from '$lib/components/loading-logo.svelte'
  import type { UnwrapReadable } from '$lib/utilities/types.ts'
  import { paginatedTransfers } from './paginated-transfers.ts'
  import { toPrettyDateTimeFormat } from '$lib/utilities/date.ts'
  import { goto, pushState, replaceState } from '$app/navigation'
  import { derived, writable, type Readable } from 'svelte/store'
  import CellDuration from '../(components)/cell-duration-text.svelte'
  import { createQuery, keepPreviousData } from '@tanstack/svelte-query'
  import CellOriginTransfer from '../(components)/cell-origin-transfer.svelte'
  import { ExplorerPagination } from '../(components)/explorer-pagination/index.ts'

  const QUERY_LIMIT = 12
  let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })
  let timestamp = writable(Temporal.Now.plainDateTimeISO('UTC').toString())

  let transfers = createQuery(
    derived([timestamp, pagination], ([$timestamp, $pagination]) => ({
      queryKey: ['transfers', $timestamp],
      staleTime: 5_000,
      placeholderData: keepPreviousData,
      refetchOnMount: $pagination.pageIndex === 0,
      refetchOnReconnect: $pagination.pageIndex === 0,
      refetchInterval: () => ($pagination.pageIndex === 0 ? 5_000 : false),
      queryFn: async () => await paginatedTransfers({ limit: QUERY_LIMIT, timestamp: $timestamp }),
    })),
  )

  let queryStatus: 'pending' | 'done' =
    $transfers.status === 'pending' || $transfers.fetchStatus === 'fetching' ? 'pending' : 'done'
  $: queryStatus =
    $transfers.status === 'pending' || $transfers.fetchStatus === 'fetching' ? 'pending' : 'done'

  let transfersDataStore = derived(transfers, $transfers => $transfers?.data ?? [])

  type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

  let timestamps = derived(transfers, $transfers => ({
    oldestTimestamp: $transfers?.data?.at(-1)?.timestamp ?? '',
    latestTimestamp: $transfers?.data?.at(0)?.timestamp ?? '',
  }))

  const encodeTimestampSearchParam = (timestamp: string) =>
    `?timestamp=${toPrettyDateTimeFormat(timestamp)?.replaceAll('-', '').replaceAll(':', '').replaceAll(' ', '')}`

  const decodeTimestampSearchParam = (search: string) => {
    const timestamp = new URLSearchParams(search).get('timestamp')
    return timestamp
      ? toPrettyDateTimeFormat(timestamp)
      : Temporal.Now.plainDateTimeISO().toString()
  }

  const unsubscribe = timestamp.subscribe(value => {
    goto(encodeTimestampSearchParam(value))
  })

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
      cell: info => toPrettyDateTimeFormat(info.getValue()),
    },
  ]

  const options = writable<TableOptions<DataRow>>({
    data: $transfersDataStore,
    enableHiding: true,
    enableFilters: true,
    columns,
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
    debugTable: import.meta.env.MODE === 'development',
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
    unsubscribe()
  })
</script>

<DevTools />
{#if $transfers.data}
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
          {@const containsAsset = $rows[row.index].original.assets}
          {@const isSupported = hasInfoProperty($rows[row.index]?.original?.assets)}
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
<ExplorerPagination
  status={queryStatus}
  totalTableRows={2000}
  timestamp={$timestamp}
  bind:rowsPerPage={$pagination.pageSize}
  onOlderPage={page => {
    timestamp.set($timestamps.oldestTimestamp)
    pagination.update(p => ({ ...p, pageIndex: p.pageIndex + 1 }))
  }}
  onNewerPage={page => {
    timestamp.set($timestamps.latestTimestamp)
    pagination.update(p => ({ ...p, pageIndex: p.pageIndex - 1 }))
  }}
/>
