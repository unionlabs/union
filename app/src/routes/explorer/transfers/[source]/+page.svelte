<script>
import { page } from "$app/stores"
import request from "graphql-request"
import { transfersBySourceHashQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"

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

  <main class="flex flex-col gap-8">
    <section class="flex">
      <div class="flex-1 lex-col text-muted-foreground">
        <h2 class="font-gunship text-2xl text-foreground">{transfer.source_chain?.display_name}</h2>
        <p>{transfer.source_chain_id}</p>
        <p>{transfer.source_connection_id}</p>
        <p>{transfer.source_channel_id}</p>
      </div>
      <div class="flex items-center justify-center">
        <div>➡️</div>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="font-gunship text-2xl text-foreground">{transfer.destination_chain?.display_name}</h2>
        <p>{transfer.destination_chain_id}</p>
        <p>{transfer.destination_connection_id}</p>
        <p>{transfer.destination_channel_id}</p>
      </div>
    </section>
    <section class="flex">
      <div class="flex-1 lex-col text-muted-foreground">
        <h2 class="font-gunship text-2xl text-foreground">Sender</h2>
        <p>{transfer.sender}</p>
        <p class="text-xs">{transfer.normalized_sender}</p>
      </div>
      <div class="flex items-center justify-center">
        <div>➡️</div>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="font-gunship text-2xl text-foreground">Receiver</h2>
        <p>{transfer.receiver}</p>
        <p class="text-xs">{transfer.normalized_receiver}</p>
      </div>
    </section>
    <section>
      <h2 class="font-gunship text-2xl text-foreground">Assets</h2>
      {#if transfer.assets}
        {#each Object.entries(transfer.assets) as [denom, value]}
          {value.amount} {denom}
        {/each}
      {:else}
        No assets in transfer
      {/if}
    </section>
    <section class="flex flex-col gap-4">
      <h2 class="font-gunship text-2xl text-foreground">Trace</h2>

      <div class="flex flex-col gap-4">
        {#each transfer.traces as trace}
          <div>
            <h3 class="text-lg font-gunship capitalize">{trace.type?.replace('_', ' ').toLowerCase()}</h3>
            <p class="text-muted-foreground">{trace.chain?.display_name}</p>
            <p class="text-muted-foreground">{trace.timestamp}</p>
            <p class="text-muted-foreground">{trace.transaction_hash}</p>
          </div>
        {/each}
      </div>
    </section>
  </main>
  {/each}
{/if}

