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
  import { page } from '$app/stores'
  import { URLS } from '$lib/constants'
  import request from 'graphql-request'
  import { raise } from '$lib/utilities'
  import { onDestroy, onMount } from 'svelte'
  import { cn } from '$lib/utilities/shadcn.ts'
  import * as Table from '$lib/components/ui/table'
  import { showUnsupported } from '$lib/stores/user.ts'
  import DevTools from '$lib/components/dev-tools.svelte'
  import * as Card from '$lib/components/ui/card/index.ts'
  import CellAssets from '../(components)/cell-assets.svelte'
  import LoadingLogo from '$lib/components/loading-logo.svelte'
  import type { UnwrapReadable } from '$lib/utilities/types.ts'
  import { toPrettyDateTimeFormat } from '$lib/utilities/date.ts'
  import { goto, pushState, replaceState } from '$app/navigation'
  import { derived, writable, type Readable } from 'svelte/store'
  import CellDuration from '../(components)/cell-duration-text.svelte'
  import CellOriginTransfer from '../(components)/cell-origin-transfer.svelte'
  import { ExplorerPagination } from '../(components)/explorer-pagination/index.ts'
  import { createQuery, useQueryClient, keepPreviousData } from '@tanstack/svelte-query'
  import { transfersTimestampFilterQueryDocument } from '$lib/graphql/documents/transfers.ts'

  const queryClient = useQueryClient()

  const QUERY_LIMIT = 12

  let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })

  // update every 5 seconds
  let timestamp = writable(Temporal.Now.plainDateTimeISO().toString(), set => {
    const interval = setInterval(() => {
      set(Temporal.Now.plainDateTimeISO().toString())
    }, 5_000)
    return () => clearInterval(interval)
  })
  let enableRefetch = $pagination.pageIndex === 0

  let transfers = createQuery(
    derived(pagination, $pagination => ({
      staleTime: 5_000,
      queryKey: ['transfers', $pagination.pageIndex],
      refetchOnMount: enableRefetch,
      refetchOnReconnect: enableRefetch,
      refetchOnWindowFocus: enableRefetch,
      refetchInterval: 5_000,
      placeholderData: keepPreviousData,
      queryFn: async () => {
        const data = await request(URLS.GRAPHQL, transfersTimestampFilterQueryDocument, {
          timestamp: {
            // if first page, use current time
            // if not, use the last timestamp
            [$pagination.pageIndex === 0 ? '_gte' : '_gt']: $timestamp,
          },
          limit: $pagination.pageSize,
          offset: $pagination.pageIndex * $pagination.pageSize,
        })

        if (!data.v0_transfers) raise('error fetching transfers')

        return data.v0_transfers.map(tx => {
          let destinationChainId = tx.destination_chain?.chain_id
          let receiver = tx.receiver

          const lastForward = tx.forwards_2?.at(-1)
          if (lastForward && lastForward.receiver !== null && lastForward.chain !== null) {
            receiver = lastForward.receiver
            destinationChainId = lastForward.chain.chain_id
          }

          return {
            source: {
              chain_display_name: tx.source_chain?.chain_id,
              address: tx.sender || 'unknown',
            },
            destination: {
              chain_display_name: tx.destination_chain?.chain_id,
              address: tx.receiver || 'unknown',
            },
            assets: tx.assets,
            timestamp: String(tx.source_timestamp),
            source_transaction_hash: tx.source_transaction_hash,
          }
        })
      },
    })),
  )

  let queryStatus: 'pending' | 'done' =
    $transfers.status === 'pending' || $transfers.fetchStatus === 'fetching' ? 'pending' : 'done'
  $: queryStatus =
    $transfers.status === 'pending' || $transfers.fetchStatus === 'fetching' ? 'pending' : 'done'

  let transfersDataStore = derived(transfers, $transfers => $transfers.data ?? [])

  type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

  let nextTimestamp = derived([transfers, timestamp], ([$transfers, $timestamp]) => {
    if ($transfers?.data && $transfers.data.length > 0) {
      console.info('last timestamp', $transfers?.data?.at(0)?.timestamp)
      return Temporal.PlainDateTime.from(String($transfers?.data?.at(0)?.timestamp)).toString()
    }
    return $timestamp
  })

  const encodeTimestampSearchParam = (timestamp: string) =>
    `?timestamp=${toPrettyDateTimeFormat(timestamp)?.replaceAll('-', '').replaceAll(':', '').replaceAll(' ', '-')}`

  const decodeTimestampSearchParam = (search: string) => {
    const timestamp = new URLSearchParams(search).get('timestamp')
    return timestamp
      ? toPrettyDateTimeFormat(timestamp)
      : Temporal.Now.plainDateTimeISO().toString()
  }

  nextTimestamp.subscribe(value => {
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
      cell: info => toPrettyDateTimeFormat(info.cell.getValue()),
    },
  ]

  const options = writable<TableOptions<DataRow>>({
    data: $transfersDataStore,
    // enableHiding: true,
    // enableFilters: true,
    columns,
    rowCount: $transfersDataStore.length,
    autoResetPageIndex: true,
    // enableColumnFilters: true,
    // enableColumnResizing: true,
    // enableMultiRowSelection: true,
    getCoreRowModel: getCoreRowModel(),
    // getFilteredRowModel: getFilteredRowModel(),
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
</script>

<DevTools />
<pre class="text-xs">{JSON.stringify(
    {
      lastRowFetched: {
        timestamp: $transfersDataStore?.at(-1)?.timestamp,
        hash: $transfersDataStore?.at(-1)?.source_transaction_hash,
      },
      config: { $pagination, status: queryStatus },
    },
    undefined,
    2,
  )}</pre>
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
        <!-- {@const containsAsset = $rows[row.index].original.assets} -->
        <!-- {#if containsAsset} -->
        {@const isSupported = hasInfoProperty($rows[row.index]?.original?.assets)}
        <!-- {#if $showUnsupported || isSupported} -->
        <Table.Row
          class={cn(
            'cursor-pointer tabular-nums',
            index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
            isSupported ? '' : 'opacity-50',
          )}
          on:click={() =>
            goto(`/explorer/transfers/${$rows[row.index].original.source_transaction_hash}`)}
        >
          {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
            <Table.Cell class="tabular-nums">
              <svelte:component this={flexRender(cell.column.columnDef.cell, cell.getContext())} />
            </Table.Cell>
          {/each}
        </Table.Row>
        <!-- {/if} -->
        <!-- {:else}
          <Table.Row
            class={cn('cursor-pointer', index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent')}
            on:click={() =>
              goto(`/explorer/transfers/${$rows[row.index].original.source_transaction_hash}`)}
          >
            {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
              <Table.Cell>
                <svelte:component
                  this={flexRender(cell.column.columnDef.cell, cell.getContext())}
                />
              </Table.Cell>
            {/each}
          </Table.Row>
        {/if} -->
      {/each}
    </Table.Body>
  </Table.Root>
</Card.Root>
<ExplorerPagination
  status={queryStatus}
  totalTableRows={2000}
  timestamp={$nextTimestamp}
  bind:rowsPerPage={$pagination.pageSize}
  onNextPage={page => pagination.update(p => ({ ...p, pageIndex: page + 1 }))}
  onPreviousPage={page => pagination.update(p => ({ ...p, pageIndex: page - 1 }))}
/>
