<script lang="ts">
import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"

// Get all token errors from the store
const tokenErrors = $derived(
  Array.from(tokensStore.error.entries())
    .filter(([_, error]) => Option.isSome(error))
    .map(([chainId, error]) => ({
      chainId,
      error: error.value
    }))
)
</script>

  {#if Option.isSome(chains.error)}
    <ErrorComponent error={chains.error.value}/>
  {/if}
  
  {#each tokenErrors as { chainId, error }}
    <div class="flex flex-col items-center gap-2">
      <span class="text-sm text-zinc-500"> {chainId}:</span>
      <ErrorComponent error={error}/>
    </div>
  {/each}
