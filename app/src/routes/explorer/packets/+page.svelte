<script lang="ts">
  import request from 'graphql-request'
  import { packetsQuery } from '$lib/graphql/documents/packets.ts'
  import { createQuery } from '@tanstack/svelte-query'
  import { URLS } from '$lib/constants'
  import Table from '../(components)/table.svelte'
  import { flexRender, type ColumnDef } from '@tanstack/svelte-table'
  import { writable } from 'svelte/store'
  import CellStatus from '../(components)/cell-status.svelte'
  import { DurationUnits } from 'svelte-ux'
  import CellDurationText from '../(components)/cell-duration-text.svelte'

  $: packets = createQuery({
    queryKey: ['packets'],
    refetchInterval: 5_000,
    queryFn: async () => request(URLS.GRAPHQL, packetsQuery, {}),
  })

  $: packetsData = $packets?.data?.v0_packets ?? []

  type DataRow = (typeof packetsData)[number]

  $: packetsStore = writable<Array<DataRow>>(packetsData as Array<DataRow>)
  $: if (packets) {
    packetsStore.update(packets => packets)
  }

  const columns: Array<ColumnDef<DataRow>> = [
    {
      accessorKey: 'source_chain_id',
      header: () => 'Source Chain',
      size: 200,
      cell: info => info.getValue(),
    },
    {
      accessorKey: 'source_channel_id',
      header: () => 'Source Channel',
      size: 200,
      cell: info => info.getValue(),
    },
    {
      accessorKey: 'source_port',
      header: () => 'Source Port',
      size: 200,
      cell: info => info.getValue(),
    },
    {
      accessorKey: 'destination_chain_id',
      header: () => 'Destination Chain',
      size: 200,
      cell: info => info.getValue(),
    },
    {
      accessorKey: 'destination_channel_id',
      header: () => 'Destination Channel',
      size: 200,
      cell: info => info.getValue(),
    },
    {
      accessorKey: 'destination_port_id',
      header: () => 'Destination Port',
      size: 200,
      cell: info => info.getValue(),
    },
    {
      accessorKey: 'status',
      header: () => 'Status',
      size: 200,
      cell: info =>
        flexRender(CellStatus, {
          value: info.getValue(),
        }),
    },
    {
      accessorKey: 'source_time',
      header: () => 'Source Time',
      size: 200,
      cell: info =>
        flexRender(CellDurationText, {
          totalUnits: 3,
          variant: 'short',
          minUnits: DurationUnits.Second,
          start: new Date(info.getValue() as string),
        }),
    },
    {
      accessorKey: 'destination_time',
      header: () => 'Destination Time',
      size: 200,
      cell: info =>
        flexRender(CellDurationText, {
          totalUnits: 3,
          variant: 'short',
          minUnits: DurationUnits.Second,
          start: new Date(info.getValue() as string),
        }),
    },
  ]
</script>

<Table enableFiltering={false} bind:dataStore={packetsStore} {columns} tableName="Packets" />
