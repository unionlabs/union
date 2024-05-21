<script lang="ts">
import {
  flexRender,
  getCoreRowModel,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import { Shine } from "svelte-ux"
import { writable } from "svelte/store"
import { cn } from "$lib/utilities/shadcn"
import { CHAIN_MAP } from "$lib/constants/chains"
import * as Table from "$lib/components/ui/table"
import { rankItem } from "@tanstack/match-sorter-utils"
import type { Override } from "$lib/utilities/types.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import { getContextClient, subscriptionStore } from "@urql/svelte"
import { dollarize, relativeTime } from "$lib/utilities/format.ts"
import Button from "$lib/components/ui/tabs/components/button.svelte"
import type { ColumnDef, TableOptions, FilterFn } from "@tanstack/svelte-table"
import { cosmosBlocksSubscription } from "$lib/graphql/documents/cosmos-blocks.ts"

$: cosmosBlocks = subscriptionStore({
  client: getContextClient(),
  query: cosmosBlocksSubscription,
  variables: { limit: 15 }
})

$: blocksData = $cosmosBlocks?.data?.data ?? []
type CosmosBlock = Override<(typeof blocksData)[number], { time: string }>

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
        class: "",
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
    cell: info => info.getValue(),
    header: info => "hash"
  }
]

const options = writable<TableOptions<CosmosBlock>>({
  data: blocksData as unknown as Array<CosmosBlock>,
  // data: (blocksData as unknown as Array<CosmosBlock>) ?? ([] as Array<CosmosBlock>),
  columns: defaultColumns,
  getCoreRowModel: getCoreRowModel(),
  enableMultiRowSelection: true,
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel()
})

const rerender = () => {
  options.update(options => ({ ...options, data: blocksData as unknown as Array<CosmosBlock> }))
}

const table = createSvelteTable(options)

$: if (blocksData) rerender()
$: console.log(JSON.stringify(blocksData, undefined, 2))
</script>

<main class="mb-12 mt-10 flex size-full min-size-full flex-col items-center justify-center">
  <Shine depth={4} lightColor="#a0ecfd">
    <h1
      class="~sm/md:~text-7xl/9xl font-black leading-[9rem] brightness-75 cursor-default select-none text-center mb-4"
    >
      zkGM
    </h1>
  </Shine>

  <Card.Root class="outline outline-1 outline-union-accent-400/20 size-full max-w-[1000px]">
    <Card.Content class="px-0 pb-2 size-full max-w-[95vw] mx-auto bg-black/70">
      <Table.Root class="size-full mx-auto bg-transparent rounded-md">
        <Table.Header class="outline outline-1 outline-union-accent-400/20">
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
          {#each $table.getRowModel().rows as row}
            <Table.Row
              class={cn('h-5', 'border-b-[1px] border-solid border-b-union-accent-400/10')}
            >
              {#each row.getVisibleCells() as cell}
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
    </Card.Content>
  </Card.Root>
</main>
