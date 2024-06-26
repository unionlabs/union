<script lang="ts">
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import NotConnected from "$lib/components/not-connected.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
</script>

<ChainsGate let:chains>
  <WalletGate let:connected let:userAddr>
    <div class="max-w-full w-full">
      {#if connected}
        <BalancesOverview {chains} {userAddr} {connected}/>
      {:else}
        <div class="flex w-full h-full items-center justify-center">
          <NotConnected/>
        </div>
      {/if}
    </div>
  </WalletGate>
</ChainsGate>
