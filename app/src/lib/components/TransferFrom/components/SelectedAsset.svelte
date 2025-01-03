<script lang="ts">
import type { IntentStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import type { Readable } from "svelte/store"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { Button } from "$lib/components/ui/button"
import { truncate } from "$lib/utilities/format.ts"

interface Props {
  intents: IntentStore
  context: Readable<ContextStore>
  onSelectAsset: () => void
}

export let intents: Props["intents"]
export let context: Props["context"]
export let onSelectAsset: Props["onSelectAsset"]
</script>

<Button
        type="button"
        size="sm"
        variant="outline"
        class="border-2 border-white font-bold"
        on:click={onSelectAsset}
>
  {$context.selectedAsset.symbol ? truncate($context.selectedAsset.symbol, 18) : "Select Asset"}
</Button>