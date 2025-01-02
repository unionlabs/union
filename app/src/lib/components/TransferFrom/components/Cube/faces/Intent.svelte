<script lang="ts">
  import Direction from "$lib/components/TransferFrom/components/Direction.svelte";
  import AssetDialog from "$lib/components/TransferFrom/components/AssetDialog.svelte";
  import type {Readable} from "svelte/store";
  import type {IntentStore} from "$lib/components/TransferFrom/transfer/intents.ts";
  import type {ValidationStoreAndMethods} from "$lib/components/TransferFrom/transfer/validation.ts";
  import type {ContextStore} from "$lib/components/TransferFrom/transfer/context.ts";
  import type {CubeFaces} from "$lib/components/TransferFrom/types.ts";
  import {Button} from "$lib/components/ui/button";

  interface Props {
    stores: {
      intents: IntentStore
      validation: ValidationStoreAndMethods
      context: Readable<ContextStore>
    }
    rotateTo: (face: CubeFaces) => void
  }

  export let stores: Props["stores"]
  export let rotateTo: Props["rotateTo"]

  $: ({intents, validation, context} = stores)
</script>

<div class="flex flex-col justify-between w-full h-full">
  <div class="flex flex-col gap-4">
    <Direction
            {context}
            {intents}
            getSourceChain={() => rotateTo("sourceFace")}
            getDestinationChain={() => rotateTo("destinationFace")}
    />
    <AssetDialog
            {context}
            {intents}
            onSelectAsset={() => rotateTo("assetsFace")}
    />
    <div class="flex flex-col gap-1">
      <input
              id="amount"
              type="number"
              name="amount"
              minlength={1}
              maxlength={64}
              required={true}
              disabled={false}
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
              on:input={event => intents.updateField('amount', event)}
      />
      {#if $validation.errors.amount}
        <span class="text-red-500 text-sm">{$validation.errors.amount}</span>
      {/if}
    </div>

    <div class="flex flex-col gap-1">
      <input
              type="text"
              id="receiver"
              name="receiver"
              required={true}
              disabled={false}
              autocorrect="off"
              spellcheck="false"
              autocomplete="off"
              data-field="receiver"
              class="p-1 disabled:bg-black/30 {$validation.errors.receiver ? 'border-red-500' : ''}"
              placeholder="Enter destination address"
              value={$intents.receiver}
              on:input={event => intents.updateField('receiver', event)}
      />
      {#if $validation.errors.receiver}
        <span class="text-red-500 text-sm">{$validation.errors.receiver}</span>
      {/if}
    </div>
  </div>
  <Button on:click={() => rotateTo("verifyFace")}>Transfer</Button>
</div>