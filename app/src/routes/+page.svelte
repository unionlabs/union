<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import BalancesOverview from "$lib/components/balances-overview.svelte"
import { ScrollArea } from "$lib/components/ui/scroll-area"
</script>


<div class="overflow-y-auto">
  <main class="flex flex-col w-full py-6 gap-6">
    <ChainsGate let:chains>
      <WalletGate>
        <div slot="connected" let:userAddr>
          <BalancesOverview {chains} {userAddr}/>
        </div>
        <div slot="disconnected" class="px-4">
          <Card.Root>
            <Card.Title>
              Disconnected card
            </Card.Title>
          </Card.Root>
        </div>
      </WalletGate>
    </ChainsGate>
  </main>
</div>



