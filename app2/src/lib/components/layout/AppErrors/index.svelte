<script lang="ts">
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"

import { tokenErrors, totalErrorCount } from "$lib/stores/app-errors.svelte"
import { channels } from "$lib/stores/channels.svelte"
</script>

{#if totalErrorCount() > 0}
  <h2 class="font-semibold text-red-500 text-lg mb-2">
    {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
  </h2>

  <div class="overflow-y-auto flex flex-col gap-4">
    {#if Option.isSome(chains.error)}
      <div>
        <Label>Chain Info Service</Label>
        <ErrorComponent error={chains.error.value} />
      </div>
    {/if}
    {#if Option.isSome(channels.error)}
      <div>
        <Label>Channel Info Service</Label>
        <ErrorComponent error={channels.error.value} />
      </div>
    {/if}
    {#each tokenErrors() as { chainId, error }}
      <div>
        <Label class="mb-2">Token Info Fetcher for Chain {chainId}</Label>
        <ErrorComponent error={error} />
      </div>
    {/each}
  </div>
{/if}
