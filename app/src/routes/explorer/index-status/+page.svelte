<script lang="ts">
import { voyagerQueueQuery } from "$lib/graphql/documents/voyager-queue.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"

$: indexStatus = createQuery({
  queryKey: ["index-status"],
  refetchInterval: 1_000,
  // enabled: false,
  queryFn: async () => {
    const response = await fetch(`${URLS.GRAPHQL_REST}/index_status`)
    const json = (await response.json()) as { v0_index_status: unknown }
    return json.v0_index_status
  }
})
</script>

<h1>Index Status</h1>

{#if $indexStatus?.data }
<pre class="overflow-scroll">
  {JSON.stringify($indexStatus?.data, null, 2)}
</pre>
{/if}
