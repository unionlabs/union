<script lang="ts">
  import request from 'graphql-request'
  import {
    createQuery,
    keepPreviousData,
    useHydrate,
    useIsFetching,
    hydrate,
    dehydrate,
  } from '@tanstack/svelte-query'
  import { URLS } from '$lib/constants'
  import { derived } from 'svelte/store'
  import CellAssets from '../(components)/cell-assets.svelte'
  import CellDuration from '../(components)/cell-duration-text.svelte'
  import CellOriginTransfer from '../(components)/cell-origin-transfer.svelte'
  import { goto } from '$app/navigation'
  import LoadingLogo from '$lib/components/loading-logo.svelte'
  import type { UnwrapReadable } from '$lib/utilities/types.ts'
  import { raise } from '$lib/utilities'
  import { transfersAfterTimestampQueryDocument } from '$lib/graphql/documents/transfers.ts'
  import { ExplorerPagination } from '../(components)/explorer-pagination/index.ts'
  import { onDestroy, onMount } from 'svelte'
  import {
    flexRender,
    type ColumnDef,
    getCoreRowModel,
    type TableOptions,
    createSvelteTable,
    getFilteredRowModel,
    getPaginationRowModel,
  } from '@tanstack/svelte-table'
  import type { MaybePromise } from 'valibot'
  import { writable, type Readable } from 'svelte/store'
  import { cn } from '$lib/utilities/shadcn.ts'
  import * as Table from '$lib/components/ui/table'
  import * as Card from '$lib/components/ui/card/index.ts'
  import { showUnsupported } from '$lib/stores/user.ts'

  const QUERY_LIMIT = 15

  // let pageIndex = 0
  // let pageSize = QUERY_LIMIT
  let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })
  let timestamp = writable(Temporal.Now.plainDateTimeISO().toString())

  let transfers = createQuery(
    derived(pagination, $pagination => ({
      queryKey: ['transfers-timestamp-gte', $pagination.pageIndex],
      staleTime: 5_000,
      placeholderData: keepPreviousData,
      queryFn: async () => {
        return await request(URLS.GRAPHQL, transfersAfterTimestampQueryDocument, {
          timestamp: {
            // if first page, use current time
            // if not, use the last timestamp
            [$pagination.pageIndex === 0 ? '_gte' : '_gt']: $timestamp,
          },
          limit: $pagination.pageSize,
          offset: $pagination.pageIndex * $pagination.pageSize,
        })
      },
      select: data => {
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
            timestamp: tx.source_timestamp,
            source_transaction_hash: tx.source_transaction_hash,
          }
        })
      },
    })),
  )

  let transfersDataStore = derived(transfers, $transfers => $transfers.data ?? [])

  type DataRow = UnwrapReadable<typeof transfersDataStore>[number]

  // $: if (typeof $transfersDataStore?.at(-1)?.timestamp === 'string') {
  //   $timestamp = Temporal.PlainDateTime.from(
  //     String($transfersDataStore.at(-1).timestamp),
  //   ).toString()
  // }

  let nextTimestamp = derived([transfers, timestamp], ([$transfers, $timestamp]) => {
    if ($transfers.data?.length > 0) {
      return Temporal.PlainDateTime.from(String($transfers.data.at(-1).timestamp)).toString()
    }
    return $timestamp
  })

  $: console.info($nextTimestamp)

  const columns: Array<ColumnDef<DataRow>> = [
    {
      accessorKey: 'source',
      header: () => 'Source',
      size: 200,
      cell: info => flexRender(CellOriginTransfer, { value: info.getValue() }),
    },
    {
      accessorKey: 'destination',
      header: () => 'Destination',
      size: 200,
      cell: info => flexRender(CellOriginTransfer, { value: info.getValue() }),
    },
    {
      accessorKey: 'assets',
      header: () => 'Assets',
      size: 200,
      cell: info => flexRender(CellAssets, { value: info.getValue() }),
    },
    {
      accessorKey: 'timestamp',
      header: () => 'Time',
      size: 200,
      cell: info => info.getValue(),
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
    return !!Object.values(assets)[0].info
  }

  $: if ($transfersDataStore) rerender()
</script>

<Card.Root>
  <div>
    <Table.Root>
      <Table.Header>
        {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <Table.Row>
            {#each headerGroup.headers as header (header.id)}
              <Table.Head
                colspan={header.colSpan}
                class={cn(`w-[${header.getSize()}px] whitespace-nowrap`)}
              >
                <svelte:component
                  this={flexRender(header.column.columnDef.header, header.getContext())}
                />
              </Table.Head>
            {/each}
          </Table.Row>
        {/each}
      </Table.Header>
      <Table.Body class={cn(`whitespace-nowrap h-full`)}>
        {#each $table.getRowModel().rows as row, index (row.index)}
          {@const containsAsset = $rows[row.index].original.assets}
          {#if containsAsset}
            {@const isSupported = hasInfoProperty(containsAsset)}
            {#if $showUnsupported || isSupported}
              <Table.Row
                class={cn(
                  'cursor-pointer',
                  index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
                  isSupported ? '' : 'opacity-50',
                )}
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
            {/if}
          {:else}
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
          {/if}
        {/each}
      </Table.Body>
    </Table.Root>
  </div>
</Card.Root>
<ExplorerPagination
  totalTableRows={2000}
  timestamp={$nextTimestamp}
  bind:rowsPerPage={$pagination.pageSize}
  onNextPage={page => pagination.update(p => ({ ...p, pageIndex: page + 1 }))}
  onPreviousPage={page => pagination.update(p => ({ ...p, pageIndex: page - 1 }))}
/>
