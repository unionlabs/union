<script lang="ts">
import type { Readable } from "svelte/store"
import { Button } from "$lib/components/ui/button"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { Intents } from "$lib/components/TransferFrom/transfer/types.ts"
import ChainDetails from "$lib/chain-details.svelte"

interface Props {
  rawIntents: RawIntentsStore
  intents: Intents
  validation: any
  getSourceChain: () => void
  getDestinationChain: () => void
}

export let rawIntents: Props["rawIntents"]
export let intents: Props["intents"]
export let validation: Props["validation"]
export let getSourceChain: Props["getSourceChain"]
export let getDestinationChain: Props["getDestinationChain"]
</script>

<div class="flex flex-col gap-2">
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 font-bold"
          on:click={getSourceChain}
  >
    {#if intents?.sourceChain}
      {#key intents?.sourceChain}
        <ChainDetails chain={intents.sourceChain}/>
      {/key}
    {:else}
      {$rawIntents.source
        ? $rawIntents.source
        : 'Source chain'}
    {/if}
  </Button>
  {#if validation.errors.source}
    <p class="text-red-500 text-sm">{validation.errors.source}</p>
  {/if}
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 font-bold"
          on:click={getDestinationChain}
  >
    {#if intents?.destinationChain}
      <ChainDetails chain={intents.destinationChain}/>
    {:else}
      {$rawIntents.destination
        ? $rawIntents.destination
        : 'Destination chain'}
    {/if}
  </Button>
  {#if validation.errors.destination}
    <p class="text-red-500 text-sm"> {validation.errors.destination}</p>
  {/if}
</div>
