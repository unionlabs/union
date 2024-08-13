<script lang="ts">
import Transfer from "./(components)/transfer.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<ChainsGate let:chains let:rawChains>
  <div class="w-full flex flex-col items-center">
    <Transfer {rawChains} {chains} />
  </div>
</ChainsGate>
