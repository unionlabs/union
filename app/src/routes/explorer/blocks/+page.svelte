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
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { writable } from "svelte/store"
import { cn } from "$lib/utilities/shadcn.ts"
import { CHAIN_MAP } from "$lib/constants/chains"
import * as Table from "$lib/components/ui/table"
import { createQuery } from "@tanstack/svelte-query"
import CellText from "../components/cell-text.svelte"
import { removeArrayDuplicates } from "$lib/utilities"
import type { Override } from "$lib/utilities/types.ts"
import { Shine, Duration, DurationUnits } from "svelte-ux"
import { createVirtualizer } from "@tanstack/svelte-virtual"
import Button from "$lib/components/ui/button/button.svelte"
import { cosmosBlocksQuery } from "$lib/graphql/documents/cosmos-blocks.ts"

$: cosmosBlocks = createQuery({
  queryKey: ["cosmos-blocks"],
  refetchInterval: 6_000,
  // enabled: false,
  queryFn: async () => request(URLS.GRAPHQL, cosmosBlocksQuery, { limit: 100 })
})

$: blockData = $cosmosBlocks?.data?.data ?? []

/**
 * we use this constructed type because importing the generated graphql types is too slow given the file size
 */
type CosmosBlock = Override<(typeof blockData)[0], { time: string }>

$: blocksStore = writable<Array<CosmosBlock>>(blockData as Array<CosmosBlock>)
$: if (blockData) {
  blocksStore.update(currentBlocks =>
    removeArrayDuplicates([...(blockData as Array<CosmosBlock>), ...currentBlocks], "height")
  )
}

const defaultColumns: Array<ColumnDef<CosmosBlock>> = [
  {
    accessorKey: "time",
    size: 105,
    maxSize: 105,
    meta: {
      class: "ml-1.5 justify-start"
    },
    header: info => "Time",
    cell: info =>
      flexRender(Duration, {
        totalUnits: 3,
        variant: "short",
        minUnits: DurationUnits.Second,
        start: new Date(info.getValue() as string),
        class: "pl-2 after:content-['_ago'] sm:after:content-[''] text-clip"
      })
  },
  {
    accessorKey: "height",
    header: info => "Height",
    size: 100,
    maxSize: 100,
    meta: {
      class: "w-full justify-start"
    },
    accessorFn: row => row.height,
    cell: info =>
      flexRender(Button, {
        variant: "link",
        target: "_blank",
        value: info.getValue(),
        rel: "noopener noreferrer",
        class: "hover:cursor-pointer tabular-nums lining-nums px-0 text-justify common-ligatures",
        href: `https://api.testnet.bonlulu.uno/cosmos/base/tendermint/v1beta1/blocks/${info.getValue()}`
      })
  },
  {
    accessorKey: "chain_id",
    header: info => "Chain ID",
    meta: {
      class: "w-full justify-start"
    },
    size: 100,
    maxSize: 100,
    cell: info =>
      flexRender(CellText, {
        value: CHAIN_MAP[info.getValue() as unknown as number].chainId,
        class: "min-w-[105px] text-clip"
      })
  },
  {
    accessorKey: "hash",
    meta: {
      class: "w-full justify-end"
    },
    header: info => flexRender(CellText, { value: "Hash", class: "text-right pr-3" }),
    size: 400,
    maxSize: 400,
    cell: info =>
      flexRender(Button, {
        class: "py-0 px-2.5 max-w-[600px]",
        variant: "link",
        target: "_blank",
        value: info.getValue(),
        rel: "noopener noreferrer",
        href: `https://rpc.testnet.bonlulu.uno/block_by_hash?hash=${info.getValue()}`
      })
  }
]

const options = writable<TableOptions<CosmosBlock>>({
  data: $blocksStore,
  enableHiding: true,
  enableFilters: true,
  columns: defaultColumns,
  autoResetPageIndex: true, // Automatically update pagination when data or page size changes
  enableColumnFilters: true,
  enableColumnResizing: true,
  enableMultiRowSelection: true,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel()
})

let virtualListElement: HTMLDivElement

const rerender = () =>
  options.update(options => ({ ...options, data: $blocksStore as unknown as Array<CosmosBlock> }))

const table = createSvelteTable(options)

$: blocksStore.subscribe(() => {
  if (!$blocksStore) return
  $table.setPageSize($blocksStore.length)
  rerender()
})

$: rows = $table.getRowModel().rows

$: virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
  overscan: 20,
  count: rows.length,
  estimateSize: () => 34,
  getScrollElement: () => virtualListElement
})
</script>

<div
  class="py-4 rounded-md mt-4 border-2 space-y-2 h-min w-full bg-card self-center flex justify-center"
>
  <div
    bind:this={virtualListElement}
    class={cn('rounded-md border border-secondary border-solid w-full max-w-[965px]')}
  >
    <Table.Root class={cn('size-full mx-auto rounded-md w-full')}>
      <Table.Header
        class={cn('outline outline-1 outline-secondary sticky top-0 left-0 bottom-0 z-50')}
      >
        {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <Table.Row class="font-bold text-md sticky">
            {#each headerGroup.headers as header (header.id)}
              <Table.Head
                colspan={header.colSpan}
                class={cn('text-left px-2 sticky top-0', `w-[${header.getSize()}px]`)}
              >
                {#if !header.isPlaceholder}
                  <Button
                    variant="ghost"
                    disabled={!header.column.getCanSort()}
                    on:click={header.column.getToggleSortingHandler()}
                    class={cn(
                      header.column.columnDef.meta?.class,
                      'cursor-pointer select-none capitalize px-0 hover:bg-transparent font-mono text-md',
                    )}
                  >
                    <svelte:component
                      this={flexRender(header.column.columnDef.header, header.getContext())}
                    />
                  </Button>
                {/if}
              </Table.Head>
            {/each}
          </Table.Row>
        {/each}
      </Table.Header>
      <Table.Body class={cn('relative', `h-[${$virtualizer.getTotalSize()}px] w-full`)}>
        {#each $virtualizer.getVirtualItems() as row, index (row.index)}
          <Table.Row
            class={cn(
              'h-5 text-left overflow-auto',
              'border-b-[1px] border-solid border-secondary',
              index % 2 === 0 ? 'bg-secondary/10' : 'bg-transparent',
            )}
          >
            {#each rows[row.index].getVisibleCells() as cell, index (cell.id)}
              <Table.Cell class={cn('px-2 py-0 text-left')}>
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
</div>

<style lang="postcss">
  :global(tr td:last-child) {
    text-align: right;
    font-variant-numeric: tabular-nums;
    font-variant: common-ligatures tabular-nums;
  }
</style>
