<script lang="ts">
  import type {IntentStore} from "../transfer/intents.ts";
  import type {ContextStore} from "$lib/components/TransferFrom/transfer/context.ts";
  import type {Readable} from "svelte/store";
  import {Button} from "$lib/components/ui/button";

  interface Props {
    intents: IntentStore
    context: Readable<ContextStore>
    getSourceChain: () => void
    getDestinationChain: () => void
  }

  export let intents: Props["intents"]
  export let context: Props["context"]
  export let getSourceChain: Props["getSourceChain"]
  export let getDestinationChain: Props["getDestinationChain"]
</script>

<div class="flex flex-col gap-1">
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 border-white font-bold"
          on:click={getSourceChain}
  >
    {$context?.sourceChain?.display_name.split(" ")[0]}
  </Button>
  <Button
          variant="outline"
          type="button"
          size="sm"
          class="border-2 border-white font-bold"
          on:click={getDestinationChain}
  >
    {$context?.destinationChain?.display_name.split(" ")[0] ?? "To chain"}
  </Button>
</div>
