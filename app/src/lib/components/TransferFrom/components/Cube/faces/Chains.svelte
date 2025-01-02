<script lang="ts">
  import type {IntentStore} from "$lib/components/TransferFrom/transfer/intents.ts";
  import type {ValidationStoreAndMethods} from "$lib/components/TransferFrom/transfer/validation.ts";
  import type {Readable} from "svelte/store";
  import type {ContextStore} from "$lib/components/TransferFrom/transfer/context.ts";
  import type {CubeFaces} from "$lib/components/TransferFrom/types.ts";

  interface Props {
    stores: {
      intents: IntentStore
      validation: ValidationStoreAndMethods
      context: Readable<ContextStore>
    }
    rotateTo: (face: CubeFaces) => void
    select: "source" | "destination"
  }

  export let stores: Props["stores"]
  export let rotateTo: Props["rotateTo"]
  export let select: Props["select"]

  $: ({intents, validation, context} = stores)
</script>

<h2 class="font-supermolot font-bold text-lg mb-4">Select chain</h2>
{#each $context.chains as chain}
  <button on:click={() => rotateTo("intentFace")}>{chain.display_name}</button>
{/each}