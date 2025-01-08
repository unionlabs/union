<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TransferFrom from "$lib/components/TransferFrom/index.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<ChainsGate>
  <div class="w-full flex flex-col justify-center items-center">
    <TransferFrom />
  </div>
</ChainsGate>
