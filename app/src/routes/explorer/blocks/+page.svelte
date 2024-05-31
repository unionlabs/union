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
import { DurationUnits } from "svelte-ux"
import { cn } from "$lib/utilities/shadcn.ts"
import { CHAIN_MAP } from "$lib/constants/chains"
import * as Table from "$lib/components/ui/table"
import { createQuery } from "@tanstack/svelte-query"
import { removeArrayDuplicates } from "$lib/utilities"
import type { Override } from "$lib/utilities/types.ts"
import { createVirtualizer } from "@tanstack/svelte-virtual"
import Button from "$lib/components/ui/button/button.svelte"
import CellText from "../(components)/cell-plain-text.svelte"
import CellDurationText from "../(components)/cell-duration-text.svelte"
import { cosmosBlocksQuery } from "$lib/graphql/documents/cosmos-blocks.ts"

$: cosmosBlocks = createQuery({
  queryKey: ["cosmos-blocks"],
  refetchInterval: 6_000,
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
    size: 100,
    meta: {
      /**
       * - the max possible size for the time cell text content is around ~50px. Example: `1m 30s`,
       * - so we need to set a min height here otherwise on lower rows, the text will be in 2 lines.
       */
      class: "ml-1.5 justify-start min-w-[50px]"
    },
    header: info => "Time",
    cell: info =>
      flexRender(CellDurationText, {
        totalUnits: 3,
        variant: "short",
        class: "pl-2 text-clip",
        minUnits: DurationUnits.Second,
        start: new Date(info.getValue() as string)
      })
  },
  {
    accessorKey: "height",
    header: info => "Height",
    size: 100,
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
      class: "w-full justify-start ml-1.5"
    },
    header: info => flexRender(CellText, { value: "Hash", class: "text-left" }),
    size: 1000,
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

$: {
  console.info($virtualizer?.getTotalSize(), "total size")
}

$: rows = $table.getRowModel().rows

$: virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
  overscan: 5,
  count: rows.length,
  estimateSize: () => 50,
  getScrollElement: () => virtualListElement
})
</script>

<svelte:head>
  <title>Union - Explorer</title>
</svelte:head>

<div
  bind:this={virtualListElement}
  class={cn(
    'rounded-md border border-secondary border-solid overflow-hidden w-full h-[800px] flex flex-col',
  )}
>
  <div class={cn(`h-[${$virtualizer.getTotalSize()}px] min-h-[${$virtualizer.getTotalSize()}px]`)}>
    <Table.Root class={cn('size-full mx-auto rounded-md w-full overflow-auto')}>
      <Table.Header
        class={cn('outline outline-1 outline-secondary sticky top-0 left-0 bottom-0 z-50 h-[20px]')}
      >
        {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <Table.Row class="font-bold text-md">
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
                      'cursor-pointer select-none capitalize px-0 hover:bg-transparent text-md',
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
      <div
        class={cn(
          /**
           * @TODO
           * this height calculation will need to be carefully tweaked to arrive at the best possible height
           */
          'overflow-auto h-[calc(800px-20px)] flex-1',
        )}
      >
        <Table.Body class={'size-full'}>
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
      </div>
    </Table.Root>
  </div>
</div>

<style lang="postcss">
  :global(tr td:last-child) {
    font-variant-numeric: tabular-nums;
    font-variant: common-ligatures tabular-nums;
  }

  .sticky {
    z-index: 1;
    background: #fff;
    position: absolute;
    border-bottom: 1px solid #ddd;
  }

  .sticky.active {
    position: sticky;
  }
</style>
