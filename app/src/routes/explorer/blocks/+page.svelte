<script lang="ts">
import {
  flexRender,
  type ColumnDef,
} from "@tanstack/svelte-table"
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
    accessorKey: "time",
    size: 100,
    meta: {
      class: "ml-1.5 justify-start"
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
    size: 200,
    cell: info =>
      flexRender(CellText, {
        value: CHAIN_MAP[info.getValue() as unknown as number].chainId
      })
  },
  {
    accessorKey: "hash",
    meta: {
      class: "w-full justify-start"
    },
    header: info => flexRender(CellText, { value: "Hash" }),
    size: 1000,
    cell: info =>
      flexRender(Button, {
        class: "p-0 font-mono",
        variant: "link",
        target: "_blank",
        value: info.getValue(),
        rel: "noopener noreferrer",
        href: `https://rpc.testnet.bonlulu.uno/block_by_hash?hash=${info.getValue()}`
      })
  }
] as Array<ColumnDef<CosmosBlock>>
</script>

<svelte:head>
  <title>Union - Explorer</title>
</svelte:head>

<Table columns={columns} bind:blocksStore={blocksStore}/>
{JSON.stringify($blocksStore, null, 2)}

