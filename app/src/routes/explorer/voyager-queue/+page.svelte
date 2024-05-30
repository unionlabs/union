<script lang="ts">
import { voyagerQueueQuery } from "$lib/graphql/documents/voyager-queue.ts"
import { createQuery } from "@tanstack/svelte-query"
import request from "graphql-request"
import { URLS } from "$lib/constants"

$: voyagerQueue = createQuery({
  queryKey: ["voyager-queue"],
  refetchInterval: 6_000,
  // enabled: false,
  queryFn: async () => request(URLS.GRAPHQL, voyagerQueueQuery, { limit: 10 })
})
</script>

<h1>Voyager Queue</h1>

{#if $voyagerQueue?.data?.queue }
<pre class="overflow-scroll">
  {JSON.stringify($voyagerQueue.data.queue, null, 2)}
</pre>
{/if}
