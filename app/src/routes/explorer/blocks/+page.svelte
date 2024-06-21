<script lang="ts">
  import request from 'graphql-request'
  import { URLS } from '$lib/constants'
  import { writable } from 'svelte/store'
  import Table from '../(components)/table.svelte'
  import { truncate } from '$lib/utilities/format'
  import { CHAIN_MAP } from '$lib/constants/chains'
  import { createQuery } from '@tanstack/svelte-query'
  import { removeArrayDuplicates } from '$lib/utilities'
  import type { Override } from '$lib/utilities/types.ts'
  import { rankItem } from '@tanstack/match-sorter-utils'
  import CellText from '../(components)/cell-plain-text.svelte'
  import CellDurationText from '../(components)/cell-duration-text.svelte'
  import { cosmosBlocksQuery } from '$lib/graphql/documents/cosmos-blocks.ts'
  import { flexRender, type ColumnDef, type FilterFn } from '@tanstack/svelte-table'

  $: cosmosBlocks = createQuery({
    queryKey: ['cosmos-blocks'],
    refetchInterval: 6_000,
    queryFn: async () => request(URLS.GRAPHQL, cosmosBlocksQuery, { limit: 100 }),
  })

  $: blockData = $cosmosBlocks?.data?.data ?? []

  /**
   * we use this constructed type because importing the generated graphql types is too slow given the file size
   */
  type DataRow = Override<(typeof blockData)[0], { time: string }>

  $: blocksStore = writable<Array<DataRow>>(blockData as Array<DataRow>)
  $: if (blockData) {
    blocksStore.update(currentBlocks =>
      removeArrayDuplicates([...(blockData as Array<DataRow>), ...currentBlocks], 'height'),
    )
  }

  let globalFilter = ''
  const fuzzyFilter = ((row, columnId, value, addMeta) => {
    const itemRank = rankItem(row.getValue(columnId), value)
    addMeta({ itemRank })
    return itemRank.passed
  }) satisfies FilterFn<DataRow>

  const columns: Array<ColumnDef<DataRow>> = [
    {
      accessorKey: 'chain_id',
      header: () => 'Chain ID',
      filterFn: 'includesString',
      cell: info => CHAIN_MAP[info.getValue() as unknown as number].chainId,
    },
    {
      accessorKey: 'height',
      header: () => 'Height',
      filterFn: 'includesString',
      accessorFn: row => row.height,
      cell: info => info.getValue(),
    },
    {
      accessorKey: 'time',
      header: () => 'Time',
      filterFn: 'includesString',
      accessorFn: row => row.time,
      cell: info => flexRender(CellDurationText, { value: info.getValue() }),
    },
    {
      accessorKey: 'hash',
      header: () => flexRender(CellText, { value: 'Hash' }),
      filterFn: 'includesString',
      accessorFn: row => row.hash,
      cell: info =>
        flexRender(CellText, {
          class: 'p-0 m-0 font-mono text-muted-foreground',
          value: truncate(String(info.getValue()), 12),
        }),
    },
  ]
</script>

<Table {columns} {fuzzyFilter} {globalFilter} tableName="Blocks" bind:dataStore={blocksStore} />
