<script lang="ts">
  import CopyUrlButton from "$lib/components/TransferFrom/components/CopyUrlButton.svelte"
  import ResetButton from "$lib/components/TransferFrom/components/ResetButton.svelte"
  import CollapsibleDisplay from "./CollapsibleDisplay.svelte"
  import type { IntentStore } from "../transfer/intents.ts"
  import type { ValidationStoreAndMethods } from "../transfer/validation.ts"
  import type { ContextStore } from "../transfer/context.ts"
  import type { Readable } from "svelte/store"

  interface DebugProps {
    intents: IntentStore
    validation: ValidationStoreAndMethods
    context: Readable<ContextStore>
  }

  export let intents: DebugProps["intents"]
  export let validation: DebugProps["validation"]
  export let context: DebugProps["context"]
</script>

<div class="p-4 w-full">
  <div class="p-4 bg-black w-full">
    <div class="mb-4 flex items-center gap-4">
      <CopyUrlButton/>
      <ResetButton onReset={intents.reset}/>
    </div>

    <h2 class="mb-4">TRANSFER DEBUG</h2>

    <div class="mb-4">
      <CollapsibleDisplay data={$intents} initiallyExpanded label="Raw Intents" color="text-union-accent-500"/>
    </div>

    <div class="mb-4">
      <CollapsibleDisplay data={$validation} initiallyExpanded label="Validation" color="text-red-500"/>
    </div>

    <div class="mb-4">
      <CollapsibleDisplay data={$context.userAddress} initiallyExpanded label="User Addresses" color="text-orange-500"/>
    </div>

    <div class="mb-4">
      <CollapsibleDisplay data={$context.assetInfo} label="Selected Asset Info" color="text-yellow-500"/>
    </div>

    <div class="mb-4">
      <CollapsibleDisplay data={$context.balances} label="Balances" color="text-purple-500"/>
    </div>

    <div class="mb-4">
      <CollapsibleDisplay data={$context.sourceChain} label="Source Chain" color="text-blue-500"/>
    </div>

    <div class="mb-4">
      <CollapsibleDisplay data={$context.destinationChain} label="Destination Chain" color="text-green-500"/>
    </div>

    <div class="mb-4">
      <CollapsibleDisplay data={$context.chains} label="Available Chains" color="text-indigo-500"/>
    </div>
  </div>
</div>