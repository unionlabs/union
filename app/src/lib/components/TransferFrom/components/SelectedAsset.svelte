<script lang="ts">
import type { Readable } from "svelte/store"
import { Button } from "$lib/components/ui/button"
import { truncate } from "$lib/utilities/format.ts"
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import type { ValidationStore } from "$lib/components/TransferFrom/transfer/validation.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"

interface Props {
  rawIntents: RawIntentsStore
  intents: Readable<IntentsStore>
  validation: Readable<ValidationStore>
  onSelectAsset: () => void
}

export let rawIntents: Props["rawIntents"]
export let intents: Props["intents"]
export let validation: Props["validation"]
export let onSelectAsset: Props["onSelectAsset"]
</script>

<div class="flex flex-col w-full gap-2">
  <Button
          disabled={!$intents.sourceChain}
          type="button"
          size="sm"
          variant="outline"
          class="border-2 font-bold"
          on:click={onSelectAsset}
  >
    {$intents.selectedAsset.symbol ?
      truncate($intents.selectedAsset.symbol, 18) :
      $rawIntents.asset ? truncate($rawIntents.asset, 6) :
        "Select Asset"}
  </Button>
  {#if $validation.errors.asset}
    <p class="text-red-500 text-sm">{$validation.errors.asset}</p>
  {/if}
</div>