<script lang="ts">
import BalanceFetcher from "$lib/components/balance-fetcher.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import UserBalances from "./UserBalances.svelte"
</script>

<ChainsGate let:chains>
  <BalanceFetcher {chains}>
    <UserBalances />
  </BalanceFetcher>
</ChainsGate>



