<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TransferFrom from "$lib/components/TransferFrom/index.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<ChainsGate let:chains>
  <div class="w-full flex flex-col items-start">
    <TransferFrom {chains} />
  </div>
</ChainsGate>
