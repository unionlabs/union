<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import SwapForm from "$lib/swap-form.svelte"
</script>


<ChainsGate let:chains>
  <SwapForm {chains}/>
</ChainsGate>
