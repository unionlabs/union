<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import UserBalances from "./UserBalances.svelte"
</script>

<ChainsGate let:chains>
  <UserBalances {chains} />
</ChainsGate>



