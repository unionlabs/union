<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import WalletGate from "$lib/components/wallet-gate.svelte"
import TransferForm from "./(components)/transfer-form.svelte"
import NotConnected from "$lib/components/not-connected.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<ChainsGate let:chains>
  <WalletGate let:connected let:userAddr>
    <div class="w-full flex flex-col items-center">
      <TransferForm {chains} {connected} {userAddr} />
    </div>
  </WalletGate>
</ChainsGate>
