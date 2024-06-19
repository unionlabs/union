<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"
import WalletGate from "$lib/components/wallet-gate.svelte"
import TransferForm from "./(components)/transfer-form.svelte"
import NotConnected from "$lib/components/not-connected.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<main class="overflow-auto flex items-center p-4 max-h-full sm:py-8 max-w-lg w-full mx-auto">
    <ChainsGate let:chains>
      <WalletGate>
        <div slot="connected" let:userAddr>
          <TransferForm {chains} {userAddr} />
        </div>
        <div class="flex items-center w-full" slot="disconnected">
          <NotConnected />
        </div>
      </WalletGate>
    </ChainsGate>
</main>
