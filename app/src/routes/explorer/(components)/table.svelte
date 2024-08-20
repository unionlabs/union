<script lang="ts">
import { onDestroy } from "svelte"
import { derived } from "svelte/store"
import {
  flexRender,
  type ColumnDef,
  getCoreRowModel,
  type TableOptions,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import type { MaybePromise } from "valibot"
import { writable, type Readable } from "svelte/store"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Table from "$lib/components/ui/table"
import { createVirtualizer } from "@tanstack/svelte-virtual"
import * as Card from "$lib/components/ui/card/index.ts"
import { showUnsupported } from "$lib/stores/user.ts"

type DataRow = $$Generic

export let columns: Array<ColumnDef<DataRow>>
export let dataStore: Readable<Array<DataRow>>
export let pageIndex = 0
export let pageSize = $dataStore.length
export let onClick: undefined | ((row: DataRow) => MaybePromise<void>) = undefined

const options = writable<TableOptions<any>>({
  data: $dataStore,
  enableHiding: true,
  enableFilters: true,
  columns,
  autoResetPageIndex: true,
  enableColumnFilters: true,
  enableColumnResizing: true,
  enableMultiRowSelection: true,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
  state: {
    pagination: { pageIndex, pageSize }
  }
})

let virtualListElement: HTMLDivElement

const table = createSvelteTable(options)
const rows = derived(table, $t => $t.getRowModel().rows)

$: virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
  overscan: 20,
  count: $rows.length,
  estimateSize: () => 34,
  getScrollElement: () => virtualListElement
})

const unsubscribe = dataStore.subscribe(() => {
  if (!$dataStore) return
  $table.setPageSize($dataStore.length)
  options.update(options => ({ ...options, data: $dataStore }))
})

function hasInfoProperty(assets: Object) {
  return !!Object.values(assets)[0].info
}

onDestroy(unsubscribe)
</script>

<Card.Root class="dark:bg-muted">
  <div bind:this={virtualListElement}>
    <Table.Root>
      <Table.Header>
        {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <Table.Row>
            <th aria-label="Search Engine Links"></th>
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
      <Table.Body class={cn(`h-[${$virtualizer.getTotalSize()}px]] whitespace-nowrap`)}>
        {#each $virtualizer.getVirtualItems() as row, index (row.index)}
          {@const containsAsset = $rows[row.index].original.assets}
          {#if containsAsset}
            {@const isSupported = hasInfoProperty(containsAsset)}
            {#if $showUnsupported || isSupported}
              <Table.Row
                class={cn("relative", onClick !== undefined ? 'cursor-pointer' : '',
              index % 2 === 0 ? 'bg-secondary/10 dark:bg-secondary/30 ' : 'bg-transparent',
              isSupported ? '' : 'opacity-50'

            )}
                on:click={onClick !== undefined ? (() => onClick($rows[row.index].original)) : undefined}
              >
                <td>
                  <a
                    href="https://google.com"
                    class="row-link"
                  ></a>
                </td>
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
              class={cn(onClick !== undefined ? 'cursor-pointer' : '',
              index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
            )}
              on:click={onClick !== undefined ? (() => onClick($rows[row.index].original)) : undefined}
            >
              <td>
                <a
                  href="https://google.com"
                  class="row-link"
                ></a>
              </td>
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

<style lang="postcss">
  .row-link {
    position: absolute;
    top: 0;
    left: 0;
    content: "";
    width: 100%;
    height: 100%;
  }
</style>
