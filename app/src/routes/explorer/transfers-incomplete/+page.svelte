<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TransfersIncomplete from "$lib/components/transfers-incomplete.svelte"
</script>

<ChainsGate let:chains>
  <TransfersIncomplete
    {chains}
  />
</ChainsGate>
