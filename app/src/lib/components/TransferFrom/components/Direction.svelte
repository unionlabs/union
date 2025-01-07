<script lang="ts">
import type { IntentsStore } from "../transfer/intents.ts"
import type { Readable } from "svelte/store"
import { Button } from "$lib/components/ui/button"
import type { ValidationStore } from "$lib/components/TransferFrom/transfer/validation.ts"

interface Props {
  intents: Readable<IntentsStore>
  validation: Readable<ValidationStore>
  getSourceChain: () => void
  getDestinationChain: () => void
}

export let intents: Props["intents"]
export let validation: Props["validation"]
export let getSourceChain: Props["getSourceChain"]
export let getDestinationChain: Props["getDestinationChain"]
</script>

<div class="flex flex-col gap-1">
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 font-bold"
          on:click={getSourceChain}
  >
    {$intents?.sourceChain?.display_name.split(" ")[0] ?? 'Source chain'}
  </Button>
  {#if $validation.errors.source}
    <p class="text-red-500 text-sm">{$validation.errors.source}</p>
  {/if}
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 font-bold"
          on:click={getDestinationChain}
  >
    {$intents?.destinationChain?.display_name.split(" ")[0] ?? "Destination chain"}
  </Button>
  {#if $validation.errors.destination}
    <p class="text-red-500 text-sm"> {$validation.errors.destination}</p>
  {/if}
</div>
