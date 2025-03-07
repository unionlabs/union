<script lang="ts">
import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { cn } from "$lib/utils"

// Get all token errors from the store
const tokenErrors = $derived(
  Array.from(tokensStore.error.entries())
    .filter(([_, error]) => Option.isSome(error))
    .map(([chainId, error]) => ({
      chainId,
      error: error.value
    }))
)

const totalErrors = $derived((Option.isSome(chains.error) ? 1 : 0) + tokenErrors.length)

let isExpanded = $state(false)
</script>

{#if totalErrors > 0}
  <button
    class="w-full px-4 py-2 flex items-center justify-between"
    onclick={() => isExpanded = !isExpanded}
  >
    <span class="font-semibold text-red-500">
      {totalErrors} Error{totalErrors > 1 ? "s" : ""}
    </span>
    <span class={cn(
      "transition-transform text-red-500",
      isExpanded ? "rotate-180" : ""
    )}>
      ↓
    </span>
  </button>

  {#if isExpanded}
    <div class="max-h-96 overflow-y-auto p-4 flex flex-col gap-4">
      {#if Option.isSome(chains.error)}
        <div>
          <Label>Chain Info Service</Label>
          <ErrorComponent error={chains.error.value}/>
        </div>
      {/if}
      {#each tokenErrors as { chainId, error }}
        <div>
          <Label class="mb-2">Token Info Fetcher for Chain {chainId}</Label>
          <ErrorComponent error={error}/>
        </div>
      {/each}
    </div>
  {/if}
{/if}
