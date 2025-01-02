<script lang="ts">
  import DebugBox from "$lib/components/TransferFrom/components/DebugBox/index.svelte";
  import {TRANSFER_DEBUG} from "$lib/components/TransferFrom/transfer/config.ts";
  import {createTransferStore } from "$lib/components/TransferFrom/transfer";
  import Intent from "$lib/components/TransferFrom/components/Cube/faces/Intent.svelte";
  import Chains from "$lib/components/TransferFrom/components/Cube/faces/Chains.svelte";
  import Assets from "$lib/components/TransferFrom/components/Cube/faces/Assets.svelte";
  import Transfer from "$lib/components/TransferFrom/components/Cube/faces/Transfer.svelte";
  import Cube from "$lib/components/TransferFrom/components/Cube/index.svelte";

  const stores = createTransferStore()
  let {context} = stores

  $: console.log($context)
</script>

<Cube>
  <div slot="intent" let:rotateTo class="w-full h-full">
    <Intent {stores} {rotateTo}/>
  </div>

  <div slot="source" let:rotateTo class="w-full h-full">
    <Chains {stores} {rotateTo} selected="source" />
  </div>

  <div slot="destination" let:rotateTo class="w-full h-full">
    <Chains {stores} {rotateTo} selected="destination" />
  </div>

  <div slot="assets" let:rotateTo class="w-full h-full">
    <Assets {stores} {rotateTo}/>
  </div>

  <div slot="transfer" let:rotateTo class="w-full h-full">
    <Transfer {stores} {rotateTo}/>
  </div>
</Cube>

<div class="absolute bottom-0 inset-x-0 text-center py-2">
  {#if TRANSFER_DEBUG}
    <DebugBox {stores} />
  {/if}
</div>
