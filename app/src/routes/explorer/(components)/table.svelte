<script lang="ts" generics="T extends object" >
import { derived, get } from "svelte/store"
import { onDestroy, onMount } from "svelte"
import {
  flexRender,
  type ColumnDef,
  getCoreRowModel,
  type TableOptions,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import { writable, type Readable } from "svelte/store"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Table from "$lib/components/ui/table"
import { createVirtualizer } from "@tanstack/svelte-virtual"
import * as Card from "$lib/components/ui/card/index.ts"

export let columns: Array<ColumnDef<any>>
// https://github.com/TanStack/table/issues/4241
// @ts-ignore
export let dataStore: Readable<Array<any>>
export let onClick: ((tr: unknown) => void) | undefined = undefined

const options = writable<TableOptions<any>>({
  data: $dataStore,
  enableHiding: true,
  enableFilters: true,
  // https://github.com/TanStack/table/issues/4241
  // @ts-ignore
  columns,
  autoResetPageIndex: true,
  enableColumnFilters: true,
  enableColumnResizing: true,
  enableMultiRowSelection: true,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel()
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
  options.update(options => ({ ...options, data: $dataStore as unknown as Array<T> }))
})

onDestroy(unsubscribe)
</script>

<Card.Root>
  <div bind:this={virtualListElement} >
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
      <Table.Body class={cn(`h-[${$virtualizer.getTotalSize()}px]] whitespace-nowrap`)}>
        {#each $virtualizer.getVirtualItems() as row, index (row.index)}
          <Table.Row
            class={cn(onClick !== undefined ? 'cursor-pointer' : '',
              index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
            )}
             on:click={onClick !== undefined ? (() => onClick($rows[row.index].original)) : undefined}
          >
            {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
              <Table.Cell>
                <svelte:component
                  this={flexRender(cell.column.columnDef.cell, cell.getContext())}
                />
              </Table.Cell>
            {/each}
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </div>
</Card.Root>
