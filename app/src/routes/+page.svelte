<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
import NotConnected from "$lib/components/not-connected.svelte"
</script>


<main
  class="overflow-auto flex p-4 sm:py-8 w-full">
  <ChainsGate let:chains>
    <WalletGate>
      <div let:userAddr slot="connected">
        <BalancesOverview {chains} {userAddr}/>
      </div>
      <div class="flex items-center w-full" slot="disconnected">
        <NotConnected/>
      </div>
    </WalletGate>
  </ChainsGate>
</main>




