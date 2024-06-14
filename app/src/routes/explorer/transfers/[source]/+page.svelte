<script>
import { page } from "$app/stores"
import request from "graphql-request"
import { transfersBySourceHashQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import MoveRightIcon from "virtual:icons/lucide/move-right"
import * as Card from "$lib/components/ui/card/index.ts"

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

  <Card.Root class="flex flex-col gap-8 max-w-5xl self-center mt-4">
    <Card.Header class="font-bold text-center">
      TRANSFER {transfer.source_transaction_hash}
    </Card.Header>
    <Card.Content class="flex flex-col gap-8">
    <section class="flex">
      <div class="flex-1 lex-col text-muted-foreground">
        <h2 class="font-gunship text-xl text-foreground">Source</h2>
        <p class="text-sm font-bold">{transfer.source_chain?.display_name}</p>
        <p class="text-xs">{transfer.source_chain_id}</p>
        <p class="text-xs">{transfer.source_connection_id}</p>
        <p class="text-xs">{transfer.source_channel_id}</p>
      </div>
      <div class="flex items-center justify-center px-8">
        <MoveRightIcon/>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="font-gunship text-xl text-foreground">Destination</h2>
        <p class="text-sm font-bold">{transfer.destination_chain?.display_name}</p>
        <p class="text-xs">{transfer.destination_chain_id}</p>
        <p class="text-xs">{transfer.destination_connection_id}</p>
        <p class="text-xs">{transfer.destination_channel_id}</p>
      </div>
    </section>
    <section class="flex">
      <div class="flex-1 lex-col text-muted-foreground">
        <h2 class="font-gunship text-xl text-foreground">Sender</h2>
        <p class="text-sm">{transfer.sender}</p>
        <p class="text-xs">{transfer.normalized_sender}</p>
      </div>
      <div class="flex items-center justify-center px-8">
        <MoveRightIcon/>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="font-gunship text-xl text-foreground">Receiver</h2>
        <p class="text-sm">{transfer.receiver}</p>
        <p class="text-xs">{transfer.normalized_receiver}</p>
      </div>
    </section>
    <section>
      <h2 class="font-gunship text-xl text-foreground">Assets</h2>
      {#if transfer.assets}
        <ul class="text-muted-foreground">
          {#each Object.entries(transfer.assets) as [denom, value]}
            <li>{value.amount} {denom}</li>
          {/each}
        </ul>
      {:else}
        No assets in transfer
      {/if}
    </section>
    <section class="flex flex-col gap-4">
      <h2 class="font-gunship text-xl text-foreground">Trace</h2>

      <div class="flex flex-col gap-4">
        {#each transfer.traces as trace}
          <div>
            <h3 class="text-md font-bold capitalize">{trace.type}</h3>
            <p class="text-sm text-muted-foreground">{trace.chain?.display_name}</p>
            <p class="text-sm text-muted-foreground">{trace.timestamp}</p>
            <p class="text-sm text-muted-foreground">{trace.transaction_hash}</p>
          </div>
        {/each}
      </div>
    </section>
    </Card.Content>
  </Card.Root>
  {/each}
{/if}

