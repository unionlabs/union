<script lang="ts">
import type { Readable } from "svelte/store"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { Button } from "$lib/components/ui/button"
import { truncate } from "$lib/utilities/format.ts"
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"

interface Props {
  intents: Readable<IntentsStore>
  context: Readable<ContextStore>
  onSelectAsset: () => void
}

export let intents: Props["intents"]
export let onSelectAsset: Props["onSelectAsset"]
</script>

<Button
        disabled={!$intents.sourceChain}
        type="button"
        size="sm"
        variant="outline"
        class="border-2 border-white font-bold"
        on:click={onSelectAsset}
>
  {$intents.selectedAsset.symbol ? truncate($intents.selectedAsset.symbol, 18) : "Select Asset"}
</Button>