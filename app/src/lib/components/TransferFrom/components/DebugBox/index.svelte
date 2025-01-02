<script lang="ts">
  import * as Drawer from "$lib/components/ui/drawer";
  import type {Readable} from "svelte/store"
  import type {IntentStore} from "../../transfer/intents.ts"
  import type {ValidationStoreAndMethods} from "../../transfer/validation.ts"
  import type {ContextStore} from "../../transfer/context.ts"
  import CollapsibleDisplay from "./CollapsibleDisplay.svelte"
  import CopyUrlButton from "./CopyUrlButton.svelte";
  import ResetButton from "./ResetButton.svelte";
  import {Button} from "$lib/components/ui/button";
  import {ScrollArea} from "$lib/components/ui/scroll-area";

  interface Props {
    stores: {
      intents: IntentStore
      validation: ValidationStoreAndMethods
      context: Readable<ContextStore>
    }
  }

  export let stores: Props["stores"]

  $: ({intents, validation, context} = stores)
</script>

<Drawer.Root>
  <Drawer.Trigger>Debug</Drawer.Trigger>
  <Drawer.Content class="h-[95svh]">
    <Drawer.Header>
      <div class="flex items-center gap-4">
        <CopyUrlButton/>
        <ResetButton onReset={intents.reset}/>
      </div>
    </Drawer.Header>

    <ScrollArea class="flex-1 px-4 rounded h-full">
      <div class="bg-black w-full h-full p-10">
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
    </ScrollArea>

    <Drawer.Footer>
      <Drawer.Close>Close</Drawer.Close>
    </Drawer.Footer>
  </Drawer.Content>
</Drawer.Root>