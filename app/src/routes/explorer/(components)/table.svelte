<script lang="ts">
import {
  flexRender,
  type FilterFn,
  type ColumnDef,
  getCoreRowModel,
  type TableOptions,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import { derived } from "svelte/store"
import type { MaybePromise } from "valibot"
import { cn } from "$lib/utilities/shadcn.ts"
import Search from "virtual:icons/lucide/search"
import * as Table from "$lib/components/ui/table"
import { writable, type Readable } from "svelte/store"
import * as Card from "$lib/components/ui/card/index.ts"
import Input from "$lib/components/ui/input/input.svelte"
import type { FormInputEvent } from "$lib/components/ui/input"
import { createVirtualizer, debounce } from "@tanstack/svelte-virtual"

type DataRow = $$Generic

export let tableName: string | undefined = undefined
export let globalFilter: string | undefined = undefined
export let fuzzyFilter: FilterFn<DataRow> | undefined = undefined
export let columns: Array<ColumnDef<DataRow>>
export let dataStore: Readable<Array<DataRow>>
export let rowsLength: number | undefined = undefined
export let onClick: undefined | ((row: DataRow) => MaybePromise<void>) = undefined

const options = writable<TableOptions<DataRow>>({
  columns,
  data: $dataStore,
  enableHiding: true,
  enableFilters: true,
  autoResetPageIndex: true,
  enableGlobalFilter: true,
  enableColumnFilters: true,
  enableColumnResizing: true,
  enableMultiRowSelection: true,
  globalFilterFn: fuzzyFilter,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
  filterFns: fuzzyFilter ? { fuzzy: fuzzyFilter } : undefined
})

let virtualListElement: HTMLDivElement

const table = createSvelteTable(options)
const rows = derived(table, $t => $t.getRowModel().rows)

$: rowsLength = $rows.length

$: $table.setGlobalFilter(globalFilter)

$: virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
  overscan: 20,
  count: $rows.length,
  estimateSize: () => 34,
  getScrollElement: () => virtualListElement
})

$: dataStore.subscribe(() => {
  if (!$dataStore) return
  $table.setPageSize($dataStore.length)
  options.update(options => ({ ...options, data: $dataStore }))
})
</script>

<!-- <div class="relative w-full">
  <Search class="absolute left-2.5 top-3 size-4 text-muted-foreground" />
  <Input
    type="text"
    autocorrect="off"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="off"
    on:keyup={handleKeyUp}
    bind:value={globalFilter}
    placeholder={`Search ${tableName || 'for any column'}`}
    class={cn(
      'bg-white/35',
      'pl-8 pr-2.5 border border-b-transparent w-full py-1 outline-none ring-0 ring-offset-0 ring-offset-transparent',
      'focus:outline-none focus:ring-0 focus-visible:ring-0 focus-visible:outline-none focus:ring-offset-0 focus-visible:ring-offset-0',
    )}
  />
</div> -->
<Card.Root>
  <div bind:this={virtualListElement}>
    <Table.Root>
      <Table.Header>
        {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <Table.Row>
            {#each headerGroup.headers as header (header.id)}
              {#if !header.id.endsWith('hidden')}
                <Table.Head
                  colspan={header.colSpan}
                  class={cn(`w-[${header.getSize()}px] whitespace-nowrap`)}
                >
                  <svelte:component
                    this={flexRender(header.column.columnDef.header, header.getContext())}
                  />
                </Table.Head>
              {/if}
            {/each}
          </Table.Row>
        {/each}
      </Table.Header>
      <Table.Body class={cn(`h-[${$virtualizer.getTotalSize()}px]] whitespace-nowrap`)}>
        {#each $virtualizer.getVirtualItems() as row, index (row.index)}
          <Table.Row
            class={cn(
              onClick !== undefined ? 'cursor-pointer' : '',
              index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
            )}
            on:click|once={onClick !== undefined
              ? () => onClick($rows[row.index].original)
              : undefined}
          >
            {#each $rows[row.index].getVisibleCells() as cell, index (cell.id)}
              {#if !cell.id.endsWith('hidden')}
                <Table.Cell>
                  <svelte:component
                    this={flexRender(cell.column.columnDef.cell, cell.getContext())}
                  />
                </Table.Cell>
              {/if}
            {/each}
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </div>
</Card.Root>
