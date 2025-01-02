<script lang="ts">
import type { IntentStore } from "../transfer/intents.ts"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import type { Readable } from "svelte/store"
import { Button } from "$lib/components/ui/button"
import type { ValidationStoreAndMethods } from "$lib/components/TransferFrom/transfer/validation.ts"

interface Props {
  intents: IntentStore
  context: Readable<ContextStore>
  validation: ValidationStoreAndMethods
  getSourceChain: () => void
  getDestinationChain: () => void
}

export let intents: Props["intents"]
export let context: Props["context"]
export let validation: Props["validation"]
export let getSourceChain: Props["getSourceChain"]
export let getDestinationChain: Props["getDestinationChain"]
</script>

<div class="flex flex-col gap-1">
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 border-white font-bold"
          on:click={getSourceChain}
  >
    {$context?.sourceChain?.display_name.split(" ")[0]}
  </Button>
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 border-white font-bold"
          on:click={getDestinationChain}
  >
    {$context?.destinationChain?.display_name.split(" ")[0] ?? "To chain"}
  </Button>
</div>
{#if $validation.errors.destination || $validation.errors.source}
  <span class="text-red-500 text-sm">
    {#if $validation.errors.destination}
      {$validation.errors.destination}
    {/if}
    {#if $validation.errors.source}
      {#if $validation.errors.destination}<br>{/if}
      {$validation.errors.source}
    {/if}
  </span>
{/if}
