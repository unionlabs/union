<script lang="ts">
import request from "graphql-request"
import { indexStatusQuery } from "$lib/graphql/documents/index-status.ts"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"

$: indexStatus = createQuery({
  queryKey: ["index-status"],
  refetchInterval: 1_000,
  // enabled: false,
  queryFn: async () => request(URLS.GRAPHQL, indexStatusQuery, {})
})
</script>

<h1>Index Status</h1>

{#if $indexStatus?.data }
<pre class="overflow-scroll">
  {JSON.stringify($indexStatus?.data.v0_index_status, null, 2)}
</pre>
{/if}
