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
import { Shine } from "svelte-ux"
import { URLS } from "$lib/constants"
import { writable } from "svelte/store"
import { CHAIN_MAP } from "$lib/constants/chains"
import * as Table from "$lib/components/ui/table"
import { removeArrayDuplicates } from "$lib/utilities"
import { rankItem } from "@tanstack/match-sorter-utils"
import type { Override } from "$lib/utilities/types.ts"
import { cn, flyAndScale } from "$lib/utilities/shadcn.ts"
import { createVirtualizer } from "@tanstack/svelte-virtual"
import Button from "$lib/components/ui/button/button.svelte"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import DoubleArrowLeft from "virtual:icons/lucide/chevrons-left"
import DoubleArrowRight from "virtual:icons/lucide/chevrons-right"
import { dollarize, relativeTime } from "$lib/utilities/format.ts"
import { cosmosBlocksQuery } from "$lib/graphql/documents/cosmos-blocks.ts"
import { getContextClient, queryStore, subscriptionStore } from "@urql/svelte"

$: initialCosmosBlocks = queryStore({
  query: cosmosBlocksQuery,
  variables: { limit: 10 },
  client: getContextClient(),
  context: { url: URLS.GRAPHQL }
})

$: initialBlocksData = $initialCosmosBlocks?.data?.data ?? []

$: cosmosBlocks = queryStore({
  query: cosmosBlocksQuery,
  client: getContextClient(),
  context: { url: URLS.GRAPHQL },
  variables: { limit: 1 }
})

$: [blockData] = $cosmosBlocks?.data?.data ?? []
/**
 * we use this constructed type because importing the generated graphql types is too slow given the file size
 */
type CosmosBlock = Override<typeof blockData, { time: string }>

$: blocksStore = writable<Array<CosmosBlock>>(initialBlocksData as Array<CosmosBlock>)
$: if ($cosmosBlocks?.data) {
  blocksStore.update(currentBlocks =>
    removeArrayDuplicates([blockData as CosmosBlock, ...currentBlocks], "height")
  )
}
// refetch every 6 seconds
setInterval(() => {
  cosmosBlocks.reexecute({ requestPolicy: "network-only" })
}, 6_000)

const defaultColumns: Array<ColumnDef<CosmosBlock>> = [
  {
    accessorKey: "time",
    header: info => "timestamp",
    cell: info => relativeTime({ timestamp: info.getValue() as string })
  },
  {
    accessorKey: "height",
    header: info => "height",
    accessorFn: row => row.height,
    cell: info =>
      flexRender(Button, {
        variant: "link",
        class: "hover:cursor-pointer",
        href: `https://api.testnet.bonlulu.uno/cosmos/base/tendermint/v1beta1/blocks/${info.getValue()}`,
        target: "_blank",
        rel: "noopener noreferrer",
        value: info.getValue()
      })
  },
  {
    accessorKey: "chain_id",
    cell: info => CHAIN_MAP[info.getValue() as unknown as number],
    header: info => "chain_id"
  },
  {
    accessorKey: "hash",
    header: info => "hash",
    cell: info =>
      flexRender(Button, {
        variant: "link",
        class: "p-0",
        href: `https://rpc.testnet.bonlulu.uno/block_by_hash?hash=${info.getValue()}`,
        target: "_blank",
        rel: "noopener noreferrer",
        value: info.getValue()
      })
  }
]

const options = writable<TableOptions<CosmosBlock>>({
  data: $blocksStore,
  // debugTable: true,
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

<main class="mb-12 mt-10 flex size-full min-size-full flex-col items-center justify-center">
  <Shine depth={4} lightColor="#a0ecfd">
    <h1
      class="~sm/md:~text-5xl/7xl font-black leading-[9rem] brightness-75 cursor-default select-none text-center mb-4"
    >
      Union Blocks
    </h1>
  </Shine>
  <!-- {#each blocksData as block}
    <pre>{JSON.stringify(block, undefined, 2)}</pre>
  {/each} -->
  <div class="space-y-2">
    <!-- <div class="">Toolbar</div> -->
    <div
      bind:this={virtualListElement}
      class={cn(
        'rounded-md border-[1px] border-solid border-union-accent-400/30 overflow-auto',
        `size-full max-h-[calc(100vh-200px)]`,
      )}
    >
      <Table.Root class="overflow-auto size-full mx-auto bg-black/70 rounded-md max-w-[1000px]">
        <Table.Header class="outline outline-1 outline-union-accent-400/50 sticky">
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
          {#each $virtualizer.getVirtualItems() as row, index (row.index)}
            <Table.Row
              class={cn(
                'h-5',
                'border-b-[1px] border-solid border-b-union-accent-400/10',
                index % 2 === 0 ? 'bg-background' : 'border-gray-950',
              )}
            >
              {#each rows[row.index].getVisibleCells() as cell (cell.id)}
                <Table.Cell class="px-4 py-0">
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

    <!-- <div class="flex items-center justify-between px-2">
      <div class="flex-1 text-sm text-muted-foreground">300 rows</div>
      <div class="flex items-center space-x-6 lg:space-x-8">
        <div class="flex items-center space-x-2">
          <p class="text-sm font-medium">Rows per page</p>
          <Select.Root
            onSelectedChange={selected => $table.setPageSize(Number(selected?.value))}
            selected={{ value: 10, label: '10' }}
          >
            <Select.Trigger class="h-8 w-[70px]">
              <Select.Value placeholder="Select page size" />
            </Select.Trigger>
            <Select.Content
              sideOffset={8}
              transition={flyAndScale}
              class="outline outline-[1px] outline-accent"
            >
              <Select.Item value="10">10</Select.Item>
              <Select.Item value="20">20</Select.Item>
              <Select.Item value="30">30</Select.Item>
              <Select.Item value="40">40</Select.Item>
              <Select.Item value="50">50</Select.Item>
            </Select.Content>
          </Select.Root>
        </div>
        <div class="flex w-[75px] items-center justify-center text-sm font-medium">
          1 of 3
        </div>
        <div class="flex items-center space-x-2">
          <Button variant="outline" class="hidden h-8 w-8 p-0 lg:flex" on:click={() => {}}>
            <span class="sr-only">Go to first page</span>
            <DoubleArrowLeft size={15} />
          </Button>
          <Button variant="outline" class="h-8 w-8 p-0">
            <span class="sr-only">Go to previous page</span>
            <ChevronLeft size={15} />
          </Button>
          <Button variant="outline" class="h-8 w-8 p-0">
            <span class="sr-only">Go to next page</span>
            <ChevronRight size={15} />
          </Button>
          <Button variant="outline" class="hidden h-8 w-8 p-0 lg:flex">
            <span class="sr-only">Go to last page</span>
            <DoubleArrowRight size={15} />
          </Button>
        </div>
      </div>
    </div> -->
    <!-- <div class="px-2 flex justify-between">
      <p class="text-white/70 text-sm">{blocksData.length} total rows</p>
      <div class="flex">
        <p class="w-max">Rows per page</p>
        <Select.Root>
          <Select.Trigger class="px-3">
            <Select.Value placeholder="Theme" class="pr-2" />
          </Select.Trigger>
          <Select.Content class="outline-union-accent-500/50">
            <Select.Item value="light">Light</Select.Item>
            <Select.Item value="dark">Dark</Select.Item>
            <Select.Item value="system">System</Select.Item>
          </Select.Content>
        </Select.Root>
      </div>
    </div> -->
  </div>
</main>
