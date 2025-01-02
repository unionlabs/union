<script lang="ts">
  import type {IntentStore} from "$lib/components/TransferFrom/transfer/intents.ts";
  import type {ValidationStoreAndMethods} from "$lib/components/TransferFrom/transfer/validation.ts";
  import type {Readable} from "svelte/store";
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

<div class="h-full w-full flex flex-col justify-between p-4">
  <h2>Transfer</h2>
  <div class="flex flex-col gap-2">
    <Button>Confirm</Button>
    <Button variant="outline" on:click={() => rotateTo("intentFace")}>CANCEL</Button>
  </div>
</div>