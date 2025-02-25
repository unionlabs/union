<script lang="ts">
import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte"
</script>


<div class="max-w-full overflow-auto">
  {#if Option.isSome(chains.error)}
    <div class="bg-red-500 m-4 p-4 rounded overflow-auto">
      <h2 class="text-xl font-bold">Error updating chains</h2>
      <pre>{chains.error.value.message}</pre>
      <pre>{chains.error.value.cause}</pre>
      <pre>{chains.error.value.stack}</pre>
    </div>
  {:else}
    <div class="bg-emerald-500 font-bold m-4 p-4 rounded">
      Chains data is fresh
    </div>
  {/if}
</div>
