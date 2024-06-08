<script lang="ts">
import { flexRender, type ColumnDef } from "@tanstack/svelte-table"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { writable } from "svelte/store"
import { DurationUnits } from "svelte-ux"
import { CHAIN_MAP } from "$lib/constants/chains"
import { createQuery } from "@tanstack/svelte-query"
import { removeArrayDuplicates } from "$lib/utilities"
import type { Override } from "$lib/utilities/types.ts"
import Button from "$lib/components/ui/button/button.svelte"
import CellText from "../(components)/cell-plain-text.svelte"
import CellDurationText from "../(components)/cell-duration-text.svelte"
import { cosmosBlocksQuery } from "$lib/graphql/documents/cosmos-blocks.ts"

import Table from "../(components)/table.svelte"
    import { truncate } from "$lib/utilities/format";

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
const columns = [
  {
    accessorKey: "chain_id",
    header: () => "Chain ID",
    meta: {},
    size: 200,
    cell: info => CHAIN_MAP[info.getValue() as unknown as number].chainId
  },
  {
    accessorKey: "height",
    header: () => "Height",
    size: 200,
    meta: {
      class: "p-0"
    },
    accessorFn: row => row.height,
    cell: info => info.getValue()
  },
  {
    accessorKey: "time",
    size: 100,
    meta: {},
    header: () => "Time",
    cell: info =>
      flexRender(CellDurationText, {
        totalUnits: 3,
        variant: "short",
        minUnits: DurationUnits.Second,
        start: new Date(info.getValue() as string)
      })
  },
  {
    accessorKey: "hash",
    meta: {},
    header: () => flexRender(CellText, { value: "Hash" }),
    size: 200,
    cell: info =>
      flexRender(CellText, {
        class: "p-0 m-0 font-mono text-muted-foreground",
        value: truncate(info.getValue(), 12)
      })
  }
] as Array<ColumnDef<CosmosBlock>>
</script>

<Table columns={columns} bind:dataStore={blocksStore}/>

