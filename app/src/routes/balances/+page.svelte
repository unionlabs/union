<script lang="ts">
import WalletGateCosmos from "$lib/components/wallet-gate-cosmos.svelte"
import WalletGateEvm from "$lib/components/wallet-gate-evm.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
</script>

<ChainsGate >
  {#snippet children({ chains })}
    <WalletGateCosmos>
      {#snippet connected({ userAddrCosmos })}
        <WalletGateEvm  >
          {#snippet connected({ userAddrEvm })}
            <div class="max-w-full w-full"  >
              <BalancesOverview {chains} userAddr={({evm: userAddrEvm, cosmos: userAddrCosmos})}/>
            </div>
          {/snippet}
        </WalletGateEvm>
      {/snippet}
    </WalletGateCosmos>
  {/snippet}
</ChainsGate>
