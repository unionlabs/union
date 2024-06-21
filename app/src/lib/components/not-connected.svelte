<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
import { Connect } from "$lib/components/connect"
import { sepoliaStore } from "$lib/wallet/evm"
import { cosmosStore } from "$lib/wallet/cosmos"

let title: string
let text: string

$: if (
  $sepoliaStore.connectionStatus === "connected" &&
  $cosmosStore.connectionStatus === "connected"
) {
  title = "All Wallets Connected"
  text = "Both wallets are connected. You can now fully use the app."
} else if (
  $sepoliaStore.connectionStatus === "connected" ||
  $cosmosStore.connectionStatus === "connected"
) {
  title = "Connect One More Wallet"
  text =
    "One of the two required wallets is connected. Please connect the other wallet to continue."
} else {
  title = "Connect Wallets"
  text = "Please connect both wallets to continue using the app."
}
</script>

<section class="max-w-lg">
  <Card.Root>
    <Card.Header>
      <Card.Title>
        {title}
      </Card.Title>
      <Card.Description>
        {text}
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <Connect />
    </Card.Content>
  </Card.Root>
</section>
