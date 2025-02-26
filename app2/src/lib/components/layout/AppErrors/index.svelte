<script lang="ts">
import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
</script>


<div class="max-w-full overflow-auto">
  {#if Option.isSome(chains.error)}
    <ErrorComponent error={chains.error.value}/>
  {:else}
    <div class="bg-emerald-500 font-bold m-4 p-4 rounded">
      Chains data is fresh
    </div>
  {/if}
</div>
