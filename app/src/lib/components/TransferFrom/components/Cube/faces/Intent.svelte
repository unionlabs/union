<script lang="ts">
import Direction from "$lib/components/TransferFrom/components/Direction.svelte"
import SelectedAsset from "$lib/components/TransferFrom/components/SelectedAsset.svelte"
import type { Readable } from "svelte/store"
import type {
  ValidationStore,
  ValidationStoreAndMethods
} from "$lib/components/TransferFrom/transfer/validation.ts"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { Button } from "$lib/components/ui/button"
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import { Input } from "$lib/components/ui/input"

interface Props {
  stores: {
    rawIntents: RawIntentsStore
    intents: Readable<IntentsStore>
    validation: Readable<ValidationStore>
  }
  rotateTo: (face: CubeFaces) => void
}

export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]

let { rawIntents, intents, validation } = stores
</script>

<div class="flex flex-col w-full h-full ">

  <div class="text-primary p-2 flex items-center justify-between border-b-2">
    <span class="font-bold uppercase">Transfer</span>
  </div>
  <div class="flex flex-col h-full w-full justify-between p-4">
    <div class="flex flex-col gap-4">
      <Direction {intents} {validation} {rawIntents} getSourceChain={() => rotateTo("sourceFace")}
                 getDestinationChain={() => rotateTo("destinationFace")}/>
      <SelectedAsset {intents} {validation} {rawIntents} onSelectAsset={() => rotateTo("assetsFace")}/>
      <div class="flex flex-col gap-1">
        <Input
                id="amount"
                type="number"
                name="amount"
                minlength={1}
                maxlength={64}
                required={true}
                disabled={!$intents.selectedAsset.address}
                autocorrect="off"
                placeholder="0.00"
                spellcheck="false"
                autocomplete="off"
                inputmode="decimal"
                data-field="amount"
                autocapitalize="none"
                pattern="^[0-9]*[.,]?[0-9]*$"
                class="p-1 {$validation.errors.amount ? 'border-red-500' : ''}"
                value={$intents.amount}
                on:input={event => rawIntents.updateField('amount', event)}
        />
        {#if $validation.errors.amount}
          <span class="text-red-500 text-sm">{$validation.errors.amount}</span>
        {/if}
      </div>

      <div class="flex flex-col gap-1">
        <Input
                type="text"
                id="receiver"
                name="receiver"
                required={true}
                disabled={!$intents.destinationChain}
                autocorrect="off"
                spellcheck="false"
                autocomplete="off"
                data-field="receiver"
                class="p-1 disabled:bg-black/30 {$validation.errors.receiver ? 'border-red-500' : ''}"
                placeholder="Enter destination address"
                value={$intents.receiver}
                on:input={event => rawIntents.updateField('receiver', event)}
        />
        {#if $validation.errors.receiver}
          <span class="text-red-500 text-sm">{$validation.errors.receiver}</span>
        {/if}
      </div>
    </div>
    <Button
            disabled={!$validation.isValid}
            on:click={() => rotateTo("verifyFace")}>Transfer
    </Button>
  </div>
</div>