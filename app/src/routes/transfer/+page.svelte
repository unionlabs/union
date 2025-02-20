<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import BalanceFetcher from "$lib/components/balance-fetcher.svelte"
import TransferCube from "$lib/components/TransferCube/index.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<ChainsGate let:chains let:ucs03channels>
  <BalanceFetcher {chains}>
    <div class="w-full flex flex-col justify-center items-center">
      <TransferCube {chains} {ucs03channels}/>
    </div>
  </BalanceFetcher>
</ChainsGate>
