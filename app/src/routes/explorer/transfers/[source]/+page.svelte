<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TransferDetails from "$lib/components/transfer-details.svelte"
</script>

<ChainsGate >
  {#snippet children({ chains })}
    <TransferDetails {chains}/>
  {/snippet}
</ChainsGate>

