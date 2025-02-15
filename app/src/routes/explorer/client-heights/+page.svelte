<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import ClientHeights from "./client-heights.svelte"
</script>

<ChainsGate
let:chains>
  <ClientHeights
    {chains}
  />
</ChainsGate>
