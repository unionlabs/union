<script lang="ts">
import WalletGateCosmos from "$lib/components/wallet-gate-cosmos.svelte"
import WalletGateEvm from "$lib/components/wallet-gate-evm.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
</script>

<ChainsGate let:chains>
  <WalletGateCosmos>
    <WalletGateEvm slot="connected" let:userAddrCosmos>
      <div class="max-w-full w-full" slot="connected" let:userAddrEvm>
        <BalancesOverview {chains} userAddr={({evm: userAddrEvm, cosmos: userAddrCosmos})}/>
      </div>
    </WalletGateEvm>
  </WalletGateCosmos>
</ChainsGate>
