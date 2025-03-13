<script lang="ts">
import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { cn } from "$lib/utils"

import { totalErrorCount, tokenErrors } from "$lib/stores/app-errors.svelte"
</script>

{#if totalErrorCount() > 0}
  <h2 class="font-semibold text-red-500 text-lg mb-2">
    {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
  </h2>

  <div class="overflow-y-auto flex flex-col gap-4">
    {#if Option.isSome(chains.error)}
      <div>
        <Label>Chain Info Service</Label>
        <ErrorComponent error={chains.error.value}/>
      </div>
    {/if}
    {#each tokenErrors() as { chainId, error }}
      <div>
        <Label class="mb-2">Token Info Fetcher for Chain {chainId}</Label>
        <ErrorComponent error={error}/>
      </div>
    {/each}
  </div>
{/if}
