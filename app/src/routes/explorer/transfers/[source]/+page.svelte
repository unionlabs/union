<script>
import { page } from "$app/stores"
import request from "graphql-request"
import { transfersBySourceHashQueryDocument } from "$lib/graphql/documents/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import MoveRightIcon from "virtual:icons/lucide/move-right"
import * as Card from "$lib/components/ui/card/index.ts"
import { truncate } from "$lib/utilities/format"
import { toIsoString } from "$lib/utilities/date"
import LoadingLogo from "$lib/components/loading-logo.svelte"
    import { derived } from "svelte/store";

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


let processedTransfers = derived(transfers, ($transfers) => {
  if (!$transfers.isSuccess) {
    return null;
  }
  return $transfers.data.map((transfer) => {
    let tx = structuredClone(transfer);


    let hop_chain = null;
    let hop_chain_id = null;
    if (tx.hop !== null) {
      hop_chain = tx.destination_chain;
      hop_chain_id = tx.destination_chain_id;
      
      tx.destination_chain = tx.hop.destination_chain;
      tx.destination_chain_id = tx.hop.destination_chain_id;
      tx.receiver = tx.hop.receiver;
      tx.normalized_receiver = tx.hop.normalized_receiver;
      tx.traces.push.apply(tx.traces, tx.hop.traces);
      // @ts-ignore timestamp is guarnateed to be a date
      tx.traces.sort((a, b) => (new Date(a.timestamp)).getTime() - (new Date(b.timestamp)).getTime());
    }

    return {
      hop_chain,
      hop_chain_id,
      ...tx
    };
  })
});
</script>

<!--
<h1 class="font-bold text-md">Transfer for <span class="font-mono">{source}</span></h1>
<a href="/explorer/transfers">Back to all transfers </a>
!-->

{#if $transfers.isLoading}
  <LoadingLogo class="size-16"/>
{:else if $transfers.isSuccess && $processedTransfers !== null}
  {#each $processedTransfers as transfer}

    <!--
    <pre>{JSON.stringify($transfers.data, null, 2)}</pre>
    !-->

  <Card.Root class="flex flex-col divide-y max-w-8xl  justify-self-center">
    <Card.Header class="font-bold text-md text-center text-muted-foreground">
      TRANSFER {transfer.source_transaction_hash}
    </Card.Header>
    <Card.Content class="flex flex-col gap-8">

    <section class="mt-6">
      {#if transfer.assets}
        <ul class="text-foreground text-center  uppercase condenced font-bold text-4xl">
          {#each Object.entries(transfer.assets) as [denom, value]}
            <li>{value.amount} {truncate(denom, 4)}</li>
          {/each}
        </ul>
      {:else}
        No assets in transfer
      {/if}
    </section>

    <section>
    <section class="flex">
      <div class="flex-1 lex-col text-muted-foreground">
        <h2 class="font-supermolot uppercase font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap">{transfer.source_chain?.display_name}</h2>
        <p class="text-sm">{transfer.source_chain_id}</p>
        <p class="text-sm">{transfer.source_connection_id}</p>
        <p class="text-sm">{transfer.source_channel_id}</p>
      </div>
      <div class="flex items-center justify-center px-8">
        <MoveRightIcon class="text-foreground size-8"/>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="font-supermolot uppercase font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap">{transfer.destination_chain?.display_name}</h2>
        <p class="text-sm">{transfer.destination_chain_id}</p>
        <p class="text-sm">{transfer.destination_connection_id}</p>
        <p class="text-sm">{transfer.destination_channel_id}</p>
      </div>
    </section>
    {#if transfer.hop_chain}
      <div class="flex-1 text-center flex-col text-sm text-muted-foreground items-center">
        forwarded through
        <h2 class="font-supermolot uppercase font-expanded text-xl font-extrabold text-foreground whitespace-nowrap">{transfer.hop_chain.display_name}</h2>
      </div>
    {/if}
    </section>
    <section class="flex gap-8">
      <div class=" lex-col text-muted-foreground">
        <h2 class="text-lg text-foreground font-bold font-supermolot">Sender</h2>
        <p class="text-sm">{transfer.sender}</p>
        <p class="text-[10px]">normalized: {transfer.normalized_sender}</p>
      </div>
      <div class="flex-1 text-right flex-col text-muted-foreground">
        <h2 class="text-lg text-foreground font-supermolot font-bold">Receiver</h2>
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

