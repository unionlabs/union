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
import * as Table from "$lib/components/ui/table"
import { removeArrayDuplicates } from "$lib/utilities"
import { rankItem } from "@tanstack/match-sorter-utils"
import type { Override } from "$lib/utilities/types.ts"
import { cn, flyAndScale } from "$lib/utilities/shadcn.ts"
import { createVirtualizer } from "@tanstack/svelte-virtual"
import Button from "$lib/components/ui/button/button.svelte"
import { dollarize, relativeTime } from "$lib/utilities/format.ts"
import { createQuery } from "@tanstack/svelte-query"
import request from "graphql-request"
import { indexStatusQuery } from "$lib/graphql/documents/index-status"
import { URLS } from "$lib/constants"
import { CHAIN_MAP } from "$lib/constants/chains"
import { writable } from "svelte/store"

$: indexStatus = createQuery({
  queryKey: ["index-status"],
  refetchInterval: 1_000,
  queryFn: async () => request(URLS.GRAPHQL, indexStatusQuery, {})
})
$: indexStatusData = $indexStatus.data?.data ?? []

console.log(indexStatusData)

type IndexStatus = Override<(typeof indexStatusData)[0], { time: string }>

const defaultColumns: Array<ColumnDef<IndexStatus>> = [
  {
    accessorKey: "display_name",
    cell: info => info.getValue(),
    header: info => "Name"
  },
  {
    accessorKey: "time",
    header: info => "Last Update",
    // cell: info => relativeTime({ timestamp: info.getValue() as string }),
    cell: info => info.getValue()
  },
  {
    accessorKey: "chain_id",
    // cell: info => CHAIN_MAP[info.getValue() as unknown as number].chainId,
    cell: info => info.getValue(),
    header: info => "Chain ID"
  }
]

const options = writable<TableOptions<IndexStatus>>({
  data: indexStatusData as any,
  enableHiding: true,
  enableFilters: true,
  columns: defaultColumns,
  // autoResetPageIndex: true, // Automatically update pagination when data or page size changes
  // enableColumnFilters: true,
  // enableColumnResizing: true,
  // enableMultiRowSelection: true,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel()
})

const rerender = () =>
  options.update(options => {
    console.log(options.data)
    return { ...options, data: indexStatusData as unknown as Array<IndexStatus> }
  })

const table = createSvelteTable(options)

$: if ($table) rerender()
</script>

<div
  class={cn('rounded-md border-[1px] border-solid border-union-accent-400/30 overflow-auto', ``)}
>
  <Table.Root class="overflow-auto size-full mx-auto bg-black/70 rounded-md max-w-[1000px]">
    <Table.Header class="capitalize outline outline-1 outline-union-accent-400/50 sticky">
      {#each $table.getHeaderGroups() as headerGroup}
        <Table.Row class="font-bold text-md">
          {#each headerGroup.headers as header}
            <Table.Head>
              {#if !header.isPlaceholder}
                <svelte:component
                  this={flexRender(header.column.columnDef.header, header.getContext())}
                />
              {/if}
            </Table.Head>
          {/each}
        </Table.Row>
      {/each}
    </Table.Header>
    <Table.Body>
      {#each $table.getRowModel().rows as row, index (row.index)}
        {@const isEven = index % 2 === 0}
        <Table.Row
          class={cn(
            isEven ? 'bg-background' : 'border-gray-950',
            'border-b-[1px] border-solid border-b-union-accent-400/10',
          )}
        >
          {#each row.getVisibleCells() as cell (cell.id)}
            <Table.Cell class="px-4 py-2">
              <svelte:component this={flexRender(cell.column.columnDef.cell, cell.getContext())} />
            </Table.Cell>
          {/each}
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
</div>
