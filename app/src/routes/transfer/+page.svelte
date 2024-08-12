<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TransferForm from "./(components)/transfer-form.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<ChainsGate let:chains>
  <div class="w-full flex flex-col items-center">
    <TransferForm {chains}/>
  </div>
</ChainsGate>
