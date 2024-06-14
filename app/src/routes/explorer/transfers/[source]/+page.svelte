<script>
import { page } from "$app/stores"
import request from "graphql-request"
import { transfersBySourceHashQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import MoveRightIcon from "virtual:icons/lucide/move-right"
import * as Card from "$lib/components/ui/card/index.ts"
import { truncate } from "$lib/utilities/format"
import { toIsoString } from '$lib/utilities/date'

const source = $page.params.source

let transfers = createQuery({
  queryKey: ["transfers-by-source", source],
  refetchInterval: 1_000,
  queryFn: async () =>
    (
      await request(URLS.GRAPHQL, transfersBySourceHashQueryDocument, {
        source_transaction_hash: source
      })
    ).v0_transfers
})
</script>

<!--
<h1 class="font-bold text-md">Transfer for <span class="font-mono">{source}</span></h1>
<a href="/explorer/transfers">Back to all transfers </a>
!-->

{#if $transfers.isLoading}
  <div>Loading...</div>
{:else if $transfers.isSuccess}
  {#each $transfers.data as transfer}

    <!--
    <pre>{JSON.stringify($transfers.data, null, 2)}</pre>
    !-->

  <Card.Root class="flex flex-col divide-y max-w-5xl self-center mt-4">
    <Card.Header class="font-bold text-md text-center text-muted-foreground">
      TRANSFER {transfer.source_transaction_hash}
    </Card.Header>
    <Card.Content class="flex flex-col gap-8">

    <section class="mt-6">
      {#if transfer.assets}
        <ul class="text-foreground text-center font-bold text-4xl">
          {#each Object.entries(transfer.assets) as [denom, value]}
            <li>{value.amount} {truncate(denom, 4)}</li>
          {/each}
        </ul>
      {:else}
        No assets in transfer
      {/if}
    </section>
    
    <section class="flex">
      <div class="flex-1 lex-col text-muted-foreground">
        <h2 class="font-gunship text-2xl text-foreground">{transfer.source_chain?.display_name}</h2>
        <p class="text-sm">{transfer.source_chain_id}</p>
        <p class="text-sm">{transfer.source_connection_id}</p>
        <p class="text-sm">{transfer.source_channel_id}</p>
      </div>
      <div class="flex items-center justify-center px-8">
        <MoveRightIcon class="size-8"/>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="font-gunship text-2xl text-foreground">{transfer.destination_chain?.display_name}</h2>
        <p class="text-sm">{transfer.destination_chain_id}</p>
        <p class="text-sm">{transfer.destination_connection_id}</p>
        <p class="text-sm">{transfer.destination_channel_id}</p>
      </div>
    </section>
    <section class="flex gap-8">
      <div class=" lex-col text-muted-foreground">
        <h2 class=" text-md text-foreground uppercase font-bold">Sender</h2>
        <p class="text-sm">{transfer.sender}</p>
        <p class="text-[10px]">normalized: {transfer.normalized_sender}</p>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="text-md text-foreground uppercase font-bold">Receiver</h2>
        <p class="text-sm">{transfer.receiver}</p>
        <p class="text-[10px]">normalized: {transfer.normalized_receiver}</p>
      </div>
    </section>
    </Card.Content>
    <Card.Footer class="items-start flex flex-col w-full gap-4">
      <div class="mt-6 font-bold text-md">{toIsoString(new Date(transfer.source_timestamp)).split('T')[0]}</div>
      <div class="flex flex-col gap-4 w-full">
        {#each transfer.traces as trace}
          <div>
            {#if trace.timestamp}
            <p class="text-sm text-muted-foreground">{toIsoString(new Date(trace.timestamp)).split('T')[1]} on {trace.chain?.display_name}</p>
            {/if}
            <h3 class="text-md font-bold capitalize">{trace.type}</h3>
            {#if trace.transaction_hash}
              <p class="text-xs text-muted-foreground">{trace.transaction_hash}</p>
            {/if}
          </div>
        {/each}
      </div>
    </Card.Footer>
  </Card.Root>
  {/each}
{/if}

