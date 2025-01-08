<script lang="ts">
import * as Drawer from "$lib/components/ui/drawer"
import type { Readable } from "svelte/store"
import type { ValidationStoreAndMethods } from "../../transfer/validation.ts"
import type { ContextStore } from "../../transfer/context.ts"
import CollapsibleDisplay from "./CollapsibleDisplay.svelte"
import CopyUrlButton from "./CopyUrlButton.svelte"
import ResetButton from "./ResetButton.svelte"
import { ScrollArea } from "$lib/components/ui/scroll-area"
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"

interface Props {
  stores: {
    rawIntents: RawIntentsStore
    intents: Readable<IntentsStore>
    validation: ValidationStoreAndMethods
    context: Readable<ContextStore>
  }
}

export let stores: Props["stores"]

let { rawIntents, intents, validation, context } = stores
</script>

<Drawer.Root>
  <Drawer.Trigger>Debug</Drawer.Trigger>
  <Drawer.Content class="h-[95svh]">
    <Drawer.Header>
      <div class="flex items-center gap-4">
        <CopyUrlButton/>
        <ResetButton onReset={rawIntents.reset}/>
      </div>
    </Drawer.Header>

    <ScrollArea class="flex-1 px-4 rounded h-full">
      <div class="bg-black w-full h-full p-10">
        <h2 class="mb-4">TRANSFER DEBUG</h2>

        <div class="mb-4">
          <CollapsibleDisplay data={$rawIntents} initiallyExpanded label="Raw Intents" color="text-union-accent-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$validation} initiallyExpanded label="Validation" color="text-red-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$context.userAddress} initiallyExpanded label="User Addresses" color="text-orange-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$intents.selectedAsset} label="Selected Asset" color="text-yellow-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$intents.sourceAssets} label="Assets List" color="text-pink-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$context.balances} label="Balances" color="text-purple-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$intents.sourceChain} label="Source Chain" color="text-blue-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$intents.destinationChain} label="Destination Chain" color="text-green-500"/>
        </div>

        <div class="mb-4">
          <CollapsibleDisplay data={$context.chains} label="Available Chains" color="text-indigo-500"/>
        </div>
      </div>
    </ScrollArea>

    <Drawer.Footer>
      <Drawer.Close>Close</Drawer.Close>
    </Drawer.Footer>
  </Drawer.Content>
</Drawer.Root>