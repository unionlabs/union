<script>
  import { page } from '$app/stores';
  import request from "graphql-request"
  import { transfersBySourceHashQueryDocument } from "$lib/graphql/documents/transfers.ts"
  import { createQuery } from "@tanstack/svelte-query"
  import { URLS } from "$lib/constants"

  const source = $page.params.source;

  let transfers = createQuery({
    queryKey: ["transfers-by-source", source],
    refetchInterval: 1_000,
    queryFn: async () => (await request(URLS.GRAPHQL, transfersBySourceHashQueryDocument, { source_transaction_hash: source })).v0_transfers
  })

</script>

<h1 class="font-bold text-2xl">Transfer for <span class="font-mono">{source}</span></h1>
<a href="/explorer/transfers">Back to all transfers </a>

{#if $transfers.isLoading}
  <div>Loading...</div>
{:else if $transfers.isSuccess}
  <pre>{JSON.stringify($transfers.data, null, 2)}</pre>
{/if}

