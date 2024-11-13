<!-- @migration-task Error while migrating Svelte code: Expected token > -->
<script lang="ts">
  import { onDestroy } from 'svelte'
  import { derived } from 'svelte/store'
  import {
    type ColumnDef,
    getCoreRowModel,
    getFilteredRowModel,
    getPaginationRowModel,
  } from '@tanstack/svelte-table'

  import type { MaybePromise } from 'valibot'
  import { writable, type Readable } from 'svelte/store'
  import { cn } from '$lib/utilities/shadcn.ts'
  import * as Table from '$lib/components/ui/table'
  import * as Card from '$lib/components/ui/card/index.ts'
  import { showUnsupported } from '$lib/stores/user.ts'
  import {
    createSvelteTable,
    renderComponent,
    renderSnippet,
    FlexRender,
    createVirtualizer,
    createWindowVirtualizer,
  } from '$lib/components/table'

  type DataRow = $$Generic

  interface Props {
    columns: Array<ColumnDef<DataRow>>
    dataStore: Readable<Array<DataRow>>
    pageIndex?: number
    pageSize?: any
    onClick?: undefined | ((row: DataRow) => MaybePromise<void>)
  }

  let {
    columns,
    dataStore,
    pageIndex = 0,
    pageSize = $dataStore.length,
    onClick = undefined,
  }: Props = $props()

  let virtualListElement: HTMLDivElement

  const table = createSvelteTable({
    get data() {
      return $dataStore
    },
    state: {
      get pagination() {
        return { pageIndex, pageSize }
      },
    },
    columns,

    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
  })

  const rows = derived(table, $t => $t.getRowModel().rows)

  const hasUrls = derived(table, $rows => typeof $rows.at(0)?.original.url === 'string')

  let virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
    overscan: 20,
    count: $rows.length,
    estimateSize: () => 34,
    getScrollElement: () => virtualListElement,
  })

  // const unsubscribe = dataStore.subscribe(() => {
  //   if (!$dataStore) return
  //   table.setPageSize($dataStore.length)
  //   // table.options.update(options => ({ ...options, data: $dataStore }))
  // })

  function hasInfoProperty(assets: Object) {
    return !!Object.values(assets)[0].info
  }

  // onDestroy(unsubscribe)
</script>

<Card.Root class="dark:bg-muted">
  <!-- <div bind:this={virtualListElement}> -->
  <Table.Root>
    <Table.Header>
      {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
        <Table.Row>
          {#if $hasUrls}
            <th aria-label="Item Detail Links"></th>
          {/if}
          {#each headerGroup.headers as header (header.id)}
            <Table.Head
              colspan={header.colSpan}
              class={cn(`w-[${header.getSize()}px] whitespace-nowrap`)}
            >
              <FlexRender content={header.column.columnDef.header} context={header.getContext()} />
            </Table.Head>
          {/each}
        </Table.Row>
      {/each}
    </Table.Header>
    <Table.Body
      class={//
      cn(`h-[${virtualizer.getTotalSize()}px]] whitespace-nowrap`)}
    >
      {#each virtualizer.getVirtualItems() as row, index (row.index)}
        {@const url = $rows[row.index].original.url ?? undefined}

        {@const containsAsset = $rows[row.index].original.assets}
        {#if containsAsset}
          {@const isSupported = hasInfoProperty(containsAsset)}
          {#if $showUnsupported || isSupported}
            <Table.Row
              class={cn(
                'relative',
                onClick !== undefined ? 'cursor-pointer' : '',
                index % 2 === 0 ? 'bg-secondary/10 dark:bg-secondary/30 ' : 'bg-transparent',
                isSupported ? '' : 'opacity-50',
              )}
              onclick={onClick !== undefined ? () => onClick($rows[row.index].original) : undefined}
            >
              {#if $hasUrls}
                <td>
                  <a href={url} aria-label={url} class="row-link"></a>
                </td>
              {/if}
              {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
                <Table.Cell>
                  <FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
                </Table.Cell>
              {/each}
            </Table.Row>
          {/if}
        {:else}
          <Table.Row
            class={cn(
              'relative',
              onClick !== undefined ? 'cursor-pointer' : '',
              index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
            )}
            onclick={onClick !== undefined ? () => onClick($rows[row.index].original) : undefined}
          >
            {#if $hasUrls}
              <td>
                <a href={url} aria-label={url} class="row-link"></a>
              </td>
            {/if}
            {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
              <Table.Cell>
                <FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
              </Table.Cell>
            {/each}
          </Table.Row>
        {/if}
      {/each}
    </Table.Body>
  </Table.Root>
  <!-- </div> -->
</Card.Root>

<style lang="postcss">
  .row-link {
    position: absolute;
    top: 0;
    left: 0;
    content: '';
    width: 100%;
    height: 100%;
  }
</style>
