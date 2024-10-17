<script lang="ts">
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { derived } from "svelte/store"
import { raise } from "$lib/utilities"
import type { UnwrapReadable } from "$lib/utilities/types"
import { CHAIN_MAP } from "$lib/constants/chains"
import { createQuery } from "@tanstack/svelte-query"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import CellText from "$lib/components/table-cells/cell-plain-text.svelte"
import CellDurationText from "$lib/components/table-cells/cell-duration-text.svelte"
import { cosmosBlocksQuery } from "$lib/graphql/queries/cosmos-blocks.ts"
import Table from "../(components)/table.svelte"
import { truncate } from "$lib/utilities/format"

let cosmosBlocks = createQuery({
  queryKey: ["cosmos-blocks"],
  refetchInterval: 6_000,
  queryFn: async () => request(URLS().GRAPHQL, cosmosBlocksQuery, { limit: 100 }),
  select: ({ data }) => {
    if (!data) raise("No data found in cosmos blocks")
    return data.map(block => ({
      hash: block.hash,
      height: block.height,
      chain_id: block.chain_id,
      time: new Date(block.time as string).toISOString()
    }))
  }
})

let blocksDataStore = derived(cosmosBlocks, $cosmosBlocks => $cosmosBlocks.data ?? [])

type DataRow = UnwrapReadable<typeof blocksDataStore>[number]

const columns: Array<ColumnDef<DataRow>> = [
  {
    accessorKey: "chain_id",
    header: () => "Chain ID",
    cell: info => CHAIN_MAP[info.getValue() as unknown as number].chainId
  },
  {
    accessorKey: "height",
    header: () => "Height",
    accessorFn: row => row.height,
    cell: info => info.getValue()
  },
  {
    accessorKey: "time",
    header: () => "Time",
    cell: info => flexRender(CellDurationText, { value: info.getValue() })
  },
  {
    accessorKey: "hash",
    header: () => flexRender(CellText, { value: "Hash" }),
    cell: info =>
      flexRender(CellText, {
        class: "p-0 m-0 font-mono text-muted-foreground",
        // @ts-expect-error
        value: truncate(info.getValue(), 12)
      })
  }
]
</script>

{#if $cosmosBlocks.data}
  <Table bind:dataStore={blocksDataStore} {columns} />
{:else if $cosmosBlocks.isLoading}
  <LoadingLogo class="size-16" />
{:else if $cosmosBlocks.isError}
  Error fetching blocks...
{/if}
