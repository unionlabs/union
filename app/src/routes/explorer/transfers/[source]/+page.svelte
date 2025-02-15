<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TransferDetails from "$lib/components/transfer-details.svelte"
</script>


<ChainsGate let:chains>
  <section class="w-full flex justify-center">
    <TransferDetails {chains}/>
  </section>
</ChainsGate>

