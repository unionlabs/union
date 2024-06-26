<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import WalletGate from "$lib/components/wallet-gate.svelte"
import TransferForm from "./(components)/transfer-form.svelte"
import NotConnected from "$lib/components/not-connected.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>


<main class="flex flex-col items-center max-h-full py-6 px-3 sm:px-6 w-full">
  <ChainsGate let:chains>
    <WalletGate let:connected let:userAddr>
      <TransferForm {chains} {connected} {userAddr}/>
    </WalletGate>
  </ChainsGate>
</main>