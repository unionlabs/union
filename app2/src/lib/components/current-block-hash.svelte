<script lang="ts">
import { Option } from "effect"
import { block } from "$lib/stores/block.svelte"
</script>

{#if Option.isSome(block.data)}
  <div class="font-mono">
    {block.data.value.result.block_id.hash}
  </div>
  {#if Option.isSome(block.error)}
    There was an error refetching this data. the data you see is stale
    <pre class="font-mono bg-red-500">
      {JSON.stringify(block.error.value, null, 2)}
    </pre>
  {/if}
{:else}
  Loading...
  {#if Option.isSome(block.error)}
    There was an error fetching this data. We will retry automatically. Error:
    <pre class="font-mono bg-red-500">
      {JSON.stringify(block.error.value, null, 2)}
    </pre>
  {/if}
{/if}
