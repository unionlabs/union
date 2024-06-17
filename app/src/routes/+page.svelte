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
      <WalletGate let:userAddr>
        <div slot="disconnected" class="px-4">
          <p>Connect your wallets to continue</p>
        </div>
        <div>
          <BalancesOverview {chains} {userAddr}/>
        </div>
      </WalletGate>
    </ChainsGate>
  </main>
</div>



