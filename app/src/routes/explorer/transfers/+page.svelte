<script lang="ts">
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { goto } from "$app/navigation"
import { cn } from "$lib/utilities/shadcn"
import { derived, writable } from "svelte/store"
import { truncate } from "$lib/utilities/format"
import Search from "virtual:icons/lucide/search"
import Table from "../(components)/table.svelte"
import { createQuery } from "@tanstack/svelte-query"
import { rankItem } from "@tanstack/match-sorter-utils"
import CellAssets from "../(components)/cell-assets.svelte"
import type { DeepNonNullable } from "$lib/utilities/types"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import CellPlainText from "../(components)/cell-plain-text.svelte"
import CellDuration from "../(components)/cell-duration-text.svelte"
import { Input, type FormInputEvent } from "$lib/components/ui/input"
import { allTransfersQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { flexRender, type ColumnDef, type FilterFn } from "@tanstack/svelte-table"
import { transactionReceiptQueryDocument } from "$lib/graphql/documents/transaction-receipt"
import { isHex } from "viem"

let transfers = createQuery({
  queryKey: ["transfers"],
  refetchInterval: 3_000,
  queryFn: async () => {
    const data = await request(URLS.GRAPHQL, allTransfersQueryDocument, {})
    return data.v0_transfers as Array<DeepNonNullable<(typeof data.v0_transfers)[number]>>
  },
  select: transfers =>
    transfers.map(transfer => {
      let receiver = transfer.receiver
      let destinationChainId = transfer.destination_chain_id

      const lastForward = transfer.forwards?.at(-1)
      if (lastForward && lastForward.receiver !== null && lastForward.chain !== null) {
        receiver = lastForward.receiver
        destinationChainId = lastForward.chain.chain_id
      }

      const source = transfer.source_chain?.display_name
      const destination = transfer.destination_chain?.display_name

      return {
        ...transfer,
        source,
        receiver,
        destination,
        timestamp: transfer.source_timestamp
      }
    })
})

$: transfersData = $transfers?.data ?? []

type DataRow = (typeof transfersData)[number]

$: transfersStore = writable<Array<DataRow>>(transfersData)
$: transfers ? transfersStore.update(transfers => transfers) : null

$: globalFilter = ""
const fuzzyFilter = ((row, columnId, value, addMeta) => {
  const itemRank = rankItem(row.getValue(columnId), value)
  addMeta({ itemRank })
  return itemRank.passed
}) satisfies FilterFn<DataRow>

const columns: Array<ColumnDef<DataRow>> = [
  {
    size: 200,
    accessorKey: "source",
    header: () => "Source",
    filterFn: "includesString",
    accessorFn: row => row.source,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "destination",
    header: () => "Destination",
    filterFn: "includesString",
    accessorFn: row => row.destination,
    cell: info => info.getValue()
  },
  {
    size: 200,
    accessorKey: "sender",
    header: () => "Sender",
    filterFn: "includesString",
    accessorFn: row => row.sender,
    cell: info => truncate(String(info.getValue()), 8)
  },
  {
    size: 200,
    accessorKey: "receiver",
    header: () => "Receiver",
    filterFn: "includesString",
    accessorFn: row => row.receiver,
    cell: info => truncate(String(info.getValue()), 8)
  },
  {
    size: 0,
    id: "hidden",
    header: () => "",
    enableHiding: true,
    filterFn: "includesString",
    accessorKey: "source_transaction_hash",
    accessorFn: row => row.source_transaction_hash,
    cell: info =>
      flexRender(CellPlainText, {
        value: info.getValue(),
        class: "hidden invisible size-0"
      })
  },
  {
    size: 200,
    accessorKey: "assets",
    header: () => "Assets",
    filterFn: "includesString",
    accessorFn: row => row.assets,
    cell: info => flexRender(CellAssets, { value: info.getValue() })
  },
  {
    size: 200,
    accessorKey: "timestamp",
    header: () => "Time",
    filterFn: "includesString",
    accessorFn: row => row.timestamp,
    cell: info => flexRender(CellDuration, { value: info.getValue() })
  }
]

let rowsLength: number

const transferReceipt = async ({ hash }: { hash: string }) => {
  const result = await request(URLS.GRAPHQL, transactionReceiptQueryDocument, {
    hash
  })
  const data = result.data as Array<DeepNonNullable<(typeof result.data)[number]>>
  console.info(data)
  // if (data.length === 0) return console.error('No data found')
  const [transfer] = data
  const formatted: DataRow = {
    sender: transfer.sender,
    receiver: transfer.receiver,
    timestamp: transfer.source_timestamp,
    source: transfer.source_chain.display_name,
    destination: transfer.destination_chain.display_name,
    source_transaction_hash: transfer.source_transaction_hash
  }
  console.info(formatted)
  transfersStore.set([formatted])
}
// 0xe8986871954d6dbe2d33a2fce401a0aa87bdb0f1d568d259c581f022b043e8da
const handleKeyUp = async (event: FormInputEvent<KeyboardEvent>) => {
  // @ts-expect-error
  globalFilter = String(event?.target["value"])

  if (
    event.key === "Enter"
    // isHex(globalFilter) && globalFilter.length === 66
  ) {
    const result = await request(URLS.GRAPHQL, transactionReceiptQueryDocument, {
      hash: "0xe8986871954d6dbe2d33a2fce401a0aa87bdb0f1d568d259c581f022b043e8da"
    })
    const data = result.data as Array<DeepNonNullable<(typeof result.data)[number]>>

    if (data.length === 0) return console.error("No data found")
    const [transfer] = data
    const formatted: DataRow = {
      sender: transfer.sender,
      receiver: transfer.receiver,
      timestamp: transfer.source_timestamp,
      source: transfer.source_chain.display_name,
      destination: transfer.destination_chain.display_name
    }

    transfersStore.set([formatted])
  }
}
</script>

<p>{rowsLength}</p>
{#if $transfers.isLoading}
  <LoadingLogo class="size-16" />
{:else if $transfers.isSuccess}
  <div class="relative w-full">
    <Search class="absolute left-2.5 top-3 size-4 text-muted-foreground" />
    <Input
      type="text"
      autocorrect="off"
      autocomplete="off"
      spellcheck="false"
      autocapitalize="off"
      on:keyup={handleKeyUp}
      bind:value={globalFilter}
      placeholder={`Search transfers, paste transaction hash`}
      class={cn(
        'bg-white/35',
        'pl-8 pr-2.5 border border-b-transparent w-full py-1 outline-none ring-0 ring-offset-0 ring-offset-transparent',
        'focus:outline-none focus:ring-0 focus-visible:ring-0 focus-visible:outline-none focus:ring-offset-0 focus-visible:ring-offset-0',
      )}
    />
  </div>
  <Table
    {columns}
    {fuzzyFilter}
    {globalFilter}
    bind:rowsLength
    tableName="Transfers"
    bind:dataStore={transfersStore}
    onClick={x => goto(`/explorer/transfers/${x.source_transaction_hash}`)}
  />
{/if}
