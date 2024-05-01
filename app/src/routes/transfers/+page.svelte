<script lang="ts">
import { graphql } from "gql.tada"
import type { PageData } from "./$types.ts"
import * as Table from "$/lib/components/ui/table"
import { balanceQuery } from "$/lib/queries/balance"
import { dollarize, relativeTime } from "$/lib/utilities"
import * as Card from "$/lib/components/ui/card/index.ts"
import { getContextClient, queryStore } from "@urql/svelte"
import { unionTransfersQuery } from "$/lib/queries/transfers"
import Button from "$/lib/components/ui/button/button.svelte"
import DraftPageNotice from "$/lib/components/draft-page-notice.svelte"
import { truncateEvmAddress, truncateUnionAddress } from "$/lib/wallet/utilities/format.ts"

$: ibcTransfersQuery = queryStore({
  client: getContextClient(),
  query: graphql(`
      query IbcTransfersQuery {
        v0_wasm_ibc_transfers(limit: 100) {
          time
          sender
          receiver
          denom
          amount
          transaction_hash
        }
      }
    `),
  variables: {}
})

$: ibcTransfers = $ibcTransfersQuery?.data?.v0_wasm_ibc_transfers || []

let error: any

export let data: PageData

const pollingIntervalMS = 2_500
</script>

<svelte:head>
  <title>Union - Transfers</title>
</svelte:head>

<DraftPageNotice />

<main
  class="mt-16 flex min-w-full flex-col items-center space-y-6 text-white max-w-4xl mx-auto px-4"
>
  <Card.Root class="border border-solid border-accent overflow-x-scroll sm:w-auto w-[95vw]">
    <!-- <Card.Title></Card.Title> -->
    <Card.Content class="p-0 max-w-5xl mx-auto">
      <Table.Root class="max-w-5xl mx-auto bg-black/80 rounded-md">
        <!-- <Table.Caption>A list of your transfers.</Table.Caption> -->
        <Table.Header>
          <Table.Row>
            <Table.Head class="w-[100px]">hash</Table.Head>
            <Table.Head>type</Table.Head>
            <Table.Head>height</Table.Head>
            <Table.Head>age</Table.Head>
            <Table.Head class="text-center">from/to</Table.Head>
            <Table.Head class="">amount</Table.Head>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#each data.transfers as { hash, height, timestamp, type, sender, receiver, amount }, index (index)}
            <Table.Row class="border-b-[1px] border-solid border-b-zinc-600/30">
              <Table.Cell class="font-medium">
                <Button variant="link" href="/" class="m-0 p-0">{hash}</Button>
              </Table.Cell>
              <Table.Cell>{type}</Table.Cell>
              <Table.Cell>
                <Button variant="link" href="/" class="m-0 p-0">{height}</Button>
              </Table.Cell>
              <Table.Cell>{relativeTime({ timestamp })}</Table.Cell>
              <Table.Cell class="text-right">
                <Button variant="link" href="/" class="m-0 p-0">
                  {truncateUnionAddress(sender, 5)}</Button
                >
                <span class="text-accent-300 hidden xs:inline-block">/</span>
                <Button variant="link" href="/" class="m-0 p-0">
                  {truncateUnionAddress(receiver, 5)}</Button
                >
              </Table.Cell>
              <Table.Cell class="text-xs">{dollarize(amount)}</Table.Cell>
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </Card.Content>
  </Card.Root>
  <!-- {#if $userUnionTransfers?.status === 'success'}
    <Table.Root class="max-w-5xl mx-auto bg-black/70 rounded-md">
      <Table.Caption >A list of your transfers.</Table.Caption>
      <Table.Header >
        <Table.Row >
          <Table.Head class="w-[100px]">hash</Table.Head>
          <Table.Head>type</Table.Head>
          <Table.Head>block</Table.Head>
          <Table.Head>from/to</Table.Head>
          <Table.Head class="text-right">amount</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#each $userUnionTransfers.data as { height, timestamp, txhash: hash, gas_used: gasUsed, tx: { '@type': type, body: { messages: [{ sender, contract, msg: { transfer: { receiver, memo: transferMemo, channel } }, funds: [{ denom, amount }] }], memo } } }}
          <Table.Row>
            <Table.Cell class="font-medium">{hash}</Table.Cell>
            <Table.Cell>{type}</Table.Cell>
            <Table.Cell>{height}</Table.Cell>
            <Table.Cell class="text-right">
              {sender}/{receiver}
            </Table.Cell>
            <Table.Cell>{amount}</Table.Cell>
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  {/if} -->
</main>
