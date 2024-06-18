<script lang="ts">
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
</script>


<div class="overflow-y-auto">
  <main class="flex flex-col w-full p-6 gap-6">
    <ChainsGate let:chains>
      <WalletGate let:userAddr>
        <BalancesOverview {chains} {userAddr}/>
      </WalletGate>
    </ChainsGate>
  </main>
</div>



