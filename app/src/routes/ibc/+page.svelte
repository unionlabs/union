<script lang="ts">
import {
  addSortBy,
  addSubRows,
  addPagination,
  addTableFilter,
  addSelectedRows,
  addHiddenColumns,
  addResizedColumns
} from "svelte-headless-table/plugins"
import type { PageData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import { readable, derived } from "svelte/store"
import CaretSort from "svelte-radix/CaretSort.svelte"
import Label from "$lib/components/ui/label/label.svelte"
import * as Table from "$lib/components/ui/table/index.js"
import Button from "$lib/components/ui/button/button.svelte"
import DraftPageNotice from "$lib/components/draft-page-notice.svelte"
import { Render, Subscribe, createRender, createTable } from "svelte-headless-table"

export let data: PageData

const [connections, channels] = [readable(data.connections), readable(data.channels)]
const [connectionsTable, channelsTable] = [
  createTable(connections, {
    page: addPagination(),
    hide: addHiddenColumns(),
    select: addSelectedRows(),
    resize: addResizedColumns(),
    sort: addSortBy({ disableMultiSort: true }),
    filter: addTableFilter({ fn: ({ filterValue, value }) => value.includes(filterValue) })
  }),
  createTable(channels, {
    sort: addSortBy({ disableMultiSort: true }),
    page: addPagination(),
    hide: addHiddenColumns(),
    select: addSelectedRows(),
    filter: addTableFilter({ fn: ({ filterValue, value }) => value.includes(filterValue) })
  })
]

const [connectionsColumns, channelsColumns] = [
  connectionsTable.createColumns([
    connectionsTable.column({ header: "state", accessor: "state" }),
    connectionsTable.column({
      header: "versions",
      accessor: "versions"
    }),
    connectionsTable.column({ header: "client_id", accessor: "client_id" }),
    // connectionsTable.group({
    //   header: 'counterparty',
    //   columns: [
    //     connectionsTable.column({
    //       header: 'client_id',
    //       id: 'counterparty.client_id',
    //       accessor: item => item.counterparty.client_id,
    //     }),
    //     connectionsTable.column({
    //       header: 'connection_id',
    //       id: 'counterparty.connection_id',
    //       accessor: item => item.counterparty.connection_id,
    //     }),
    //   ],
    // }),
    connectionsTable.column({
      header: createRender(Label, { class: "break-all line-clamp-2" }).slot(
        `counterparty.client_id`
      ),
      accessor: data => data.counterparty.client_id
    }),
    connectionsTable.column({ header: "delay_period", accessor: "delay_period" }),
    connectionsTable.column({
      id: "full.payload",
      header: "full payload",
      accessor: item => JSON.stringify(item, undefined, 2)
    })
  ]),
  channelsTable.createColumns([
    channelsTable.column({ header: "state", accessor: "state" }),
    channelsTable.column({ header: "version", accessor: "version" }),
    channelsTable.column({ header: "port_id", accessor: "port_id" }),
    channelsTable.column({ header: "ordering", accessor: "ordering" }),
    channelsTable.column({ header: "channel_id", accessor: "channel_id" }),
    channelsTable.column({ header: "counterparty", accessor: "counterparty" }),
    channelsTable.column({ header: "connection_hops", accessor: "connection_hops" })
  ])
]

const [
  {
    rows: connectionsRows,
    pageRows: connectionsPageRows,
    headerRows: connectionsHeaderRows,
    tableAttrs: connectionsTableAttrs,
    flatColumns: connectionsFlatColumns,
    originalRows: connectionsOriginalRows,
    pluginStates: connectionsPluginStates,
    tableBodyAttrs: connectionsTableBodyAttrs,
    tableHeadAttrs: connectionsTableHeadAttrs,
    visibleColumns: connectionsVisibleColumns
  },
  {
    rows: channelsRows,
    pageRows: channelsPageRows,
    headerRows: channelsHeaderRows,
    tableAttrs: channelsTableAttrs,
    flatColumns: channelsFlatColumns,
    originalRows: channelsOriginalRows,
    pluginStates: channelsPluginStates,
    tableBodyAttrs: channelsTableBodyAttrs,
    tableHeadAttrs: channelsTableHeadAttrs,
    visibleColumns: channelsVisibleColumns
  }
] = [
  connectionsTable.createViewModel(connectionsColumns),
  channelsTable.createViewModel(channelsColumns)
]

const [{ sortKeys: sortConnectionsKeys }, { sortKeys: sortChannelsKeys }] = [
  connectionsPluginStates.sort,
  channelsPluginStates.sort
]

const [
  { hiddenColumnIds: hiddenConnectionsColumnIds },
  { hiddenColumnIds: hiddenChannelsColumnIds }
] = [connectionsPluginStates.hide, channelsPluginStates.hide]

const [connectionsIds, channelsIds] = [
  channelsFlatColumns.map(column => column.id),
  connectionsFlatColumns.map(column => column.id)
]

const [connectionsHideForId, channelsHideForId] = [
  Object.fromEntries(connectionsIds.map(id => [id, true])),
  Object.fromEntries(channelsIds.map(id => [id, true]))
]

$: $hiddenChannelsColumnIds = Object.entries(channelsHideForId)
  .filter(([, hide]) => !hide)
  .map(([id]) => id)

$: $hiddenConnectionsColumnIds = Object.entries(connectionsHideForId)
  .filter(([, hide]) => !hide)
  .map(([id]) => id)

const [
  {
    pageSize: connectionsPageSize,
    pageIndex: connectionsPageIndex,
    pageCount: connectionsPageCount,
    hasNextPage: connectionsHasNextPage,
    hasPreviousPage: connectionsHasPreviousPage
  },
  {
    pageSize: channelsPageSize,
    pageIndex: channelsPageIndex,
    pageCount: channelsPageCount,
    hasNextPage: channelsHasNextPage,
    hasPreviousPage: channelsHasPreviousPage
  }
] = [channelsPluginStates.page, connectionsPluginStates.page]

const [
  { filterValue: connectionsFilterValue, preFilteredRows: connectionsPreFilteredRows },
  { filterValue: channelsFilterValue, preFilteredRows: channelsPreFilteredRows }
] = [channelsPluginStates.filter, connectionsPluginStates.filter]

const [
  {
    selectedDataIds: selectedConnectionsDataIds,
    somePageRowsSelected: someConnectionsPageRowsSelected,
    someRowsSelected: someConnectionsRowsSelected,
    getRowState: getConnectionsRowState,
    allRowsSelected: allConnectionsRowsSelected,
    allPageRowsSelected: allConnectionsPageRowsSelected
  },
  {
    selectedDataIds: selectedChannelsDataIds,
    somePageRowsSelected: someChannelsPageRowsSelected,
    someRowsSelected: someChannelsRowsSelected,
    getRowState: getChannelsRowState,
    allRowsSelected: allChannelsRowsSelected,
    allPageRowsSelected: allChannelsPageRowsSelected
  }
] = [channelsPluginStates.select, connectionsPluginStates.select]

const hideableChannelsColumns = ["ordering", "version", "counterparty"]
</script>

<svelte:head>
  <title>Union - IBC</title>
</svelte:head>

<main
  class="size-full mx-auto flex flex-col justify-center align-middle items-center mt-4 max-w-5xl"
>
  <DraftPageNotice />
  <!-- <p class="mb-3">IBC Channels</p>
  
    <Table.Root class="bg-[#030304] size-full mx-auto mb-12">
    <Table.Caption class="mx-auto text-center w-full">A list of your recent invoices.</Table.Caption
    >
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[100px]">state</Table.Head>
        <Table.Head>ordering</Table.Head>
        <Table.Head>counterparty</Table.Head>
        <Table.Head>connection hops</Table.Head>
        <Table.Head>version</Table.Head>
        <Table.Head>port id</Table.Head>
        <Table.Head class="text-right">channel id</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each data.channels as ibcChannel, index (index)}
        <Table.Row>
          <Table.Cell class="font-medium">{ibcChannel.state}</Table.Cell>
          <Table.Cell>{ibcChannel.ordering}</Table.Cell>
          <Table.Cell>{JSON.stringify(ibcChannel.counterparty, undefined, 2)}</Table.Cell>
          <Table.Cell>{ibcChannel.connection_hops}</Table.Cell>
          <Table.Cell>{ibcChannel.version}</Table.Cell>
          <Table.Cell>{ibcChannel.port_id}</Table.Cell>
          <Table.Cell class="text-right truncate">{ibcChannel.channel_id}</Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root> -->
  <p class="mb-3">IBC Connections</p>

  <Table.Root class="bg-[#030304] size-full mx-auto mb-12">
    <!-- <Table.Caption class="mx-auto text-center w-full">IBC Connections</Table.Caption> -->
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[100px]">state</Table.Head>
        <Table.Head>versions</Table.Head>
        <Table.Head>client_id</Table.Head>
        <Table.Head>counterparty</Table.Head>
        <Table.Head>delay_period</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each data.connections as ibcConnection, index (index)}
        <Table.Row>
          <Table.Cell class="font-medium">{ibcConnection.state}</Table.Cell>
          <Table.Cell>{JSON.stringify(ibcConnection.versions, undefined, 2)}</Table.Cell>
          <Table.Cell>{ibcConnection.client_id}</Table.Cell>
          <Table.Cell>{JSON.stringify(ibcConnection.counterparty, undefined, 2)}</Table.Cell>
          <Table.Cell class="text-center">{ibcConnection.delay_period}</Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>

  <p class="mb-3">IBC Connections</p>
  <Table.Root {...$connectionsTableAttrs} class="bg-[#030304] size-full mx-auto mb-12">
    <Table.Header>
      {#each $connectionsHeaderRows as headerRow, index (index)}
        <Subscribe rowAttrs={headerRow.attrs()} rowProps={headerRow.props()} let:rowProps>
          <Table.Row>
            {#each headerRow.cells as cell (cell.id)}
              <Subscribe attrs={cell.attrs()} let:attrs props={cell.props()} let:props>
                <Table.Head {...attrs} class={cn('[&:has([role=checkbox])]:pl-3')}>
                  <Button variant="ghost" on:click={props.sort.toggle}>
                    <Render of={cell.render()} />
                    <!-- <CaretSort
                      class={cn(
                        $sortConnectionsKeys[0]?.id === cell.id && 'text-foreground',
                        'ml-2 h-4 w-4',
                      )}
                    /> -->
                  </Button>
                </Table.Head>
              </Subscribe>
            {/each}
          </Table.Row>
        </Subscribe>
      {/each}
    </Table.Header>
    <Table.Body {...$connectionsTableBodyAttrs}>
      {#each $connectionsPageRows as row (row.id)}
        <Subscribe rowAttrs={row.attrs()} let:rowAttrs>
          <Table.Row {...rowAttrs} data-state={$selectedConnectionsDataIds[row.id] && 'selected'}>
            {#each row.cells as cell (cell.id)}
              {#if cell.id === 'full.payload'}
                <Button variant="outline" size="sm" class="text-right">payload</Button>
              {:else}
                <Table.Cell class="">{cell.render()}</Table.Cell>
              {/if}
            {/each}
          </Table.Row>
        </Subscribe>
      {/each}
    </Table.Body>
  </Table.Root>
  <div>
    <Button
      size="sm"
      variant="outline"
      disabled={!$connectionsHasPreviousPage}
      on:click={() => ($connectionsPageIndex = $connectionsPageIndex - 1)}
    >
      Previous
    </Button>
    <Button
      size="sm"
      variant="outline"
      disabled={!$connectionsHasNextPage}
      on:click={() => ($connectionsPageIndex = $connectionsPageIndex + 1)}
    >
      Next
    </Button>
  </div>
</main>
