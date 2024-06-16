<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
    import { ScrollArea } from "$lib/components/ui/scroll-area";
</script>


<ScrollArea orientation="both">
  <main class="flex flex-col justify-center items-center w-full p-6 gap-6">
  <Card.Root class="max-w-xl size-full">
    <Card.Header>
      <Card.Title>Welcome to Union</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col gap-2">
      <WalletGate let:userAddr>
        Welcome to Union
        <div>Cosmos: {userAddr.cosmos.canonical}</div>
        <div>EVM: {userAddr.evm.canonical}</div>
      </WalletGate>
    </Card.Content>
  </Card.Root>

  <Card.Root class="max-w-xl w-full">
    <Card.Header>
      <Card.Title>Balances</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col gap-6">
      <ChainsGate let:chains>
        <WalletGate let:userAddr>
          <BalancesOverview {chains} {userAddr}/>
        </WalletGate>
      </ChainsGate>
    </Card.Content>
  </Card.Root>
  </main>
</ScrollArea>



