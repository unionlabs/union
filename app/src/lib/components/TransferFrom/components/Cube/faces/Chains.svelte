<script lang="ts">
import type { IntentStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import type { ValidationStoreAndMethods } from "$lib/components/TransferFrom/transfer/validation.ts"
import type { Readable } from "svelte/store"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import type { CubeFaces } from "$lib/components/TransferFrom/types.ts"
import { Button } from "$lib/components/ui/button"
import { truncate } from "$lib/utilities/format.ts"
import { formatUnits } from "viem"

interface Props {
  stores: {
    intents: IntentStore
    validation: ValidationStoreAndMethods
    context: Readable<ContextStore>
  }
  rotateTo: (face: CubeFaces) => void
  selected: "source" | "destination"
}

export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]
export let selected: Props["selected"]

$: ({ intents, validation, context } = stores)

function setChain(selected: "source" | "destination", chainId: string) {
  intents.updateField(selected, chainId)
  console.log(selected, chainId)
  rotateTo("intentFace")
}
</script>

<div class="flex h-full justify-between flex-col gap-4 p-4">
  <div class="flex flex-col gap-2">
    {#each $context.chains as chain}
      <Button variant="ghost"
              class="px-4 py-2 w-full rounded-none flex justify-between items-center"
              on:click={() => setChain(selected, chain.chain_id)}
      >{chain.display_name}
      </Button>
    {/each}
  </div>
  <Button on:click={() => rotateTo("intentFace")}>Back</Button>
</div>
