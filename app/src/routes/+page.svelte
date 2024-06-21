<script lang="ts">
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import NotConnected from "$lib/components/not-connected.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
</script>

<ChainsGate let:chains>
  <WalletGate>
    <div class="max-w-full" let:userAddr slot="connected">
      <BalancesOverview {chains} {userAddr}/>
    </div>
    <div class="flex items-center justify-center w-full h-full" slot="disconnected">
      <NotConnected/>
    </div>
  </WalletGate>
</ChainsGate>
