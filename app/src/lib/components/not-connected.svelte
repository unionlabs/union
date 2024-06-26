<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
import { Connect } from "$lib/components/connect"
import { sepoliaStore } from "$lib/wallet/evm"
import { cosmosStore } from "$lib/wallet/cosmos"

let title: string
let text: string

$: if (
  $sepoliaStore.connectionStatus !== "connected" &&
  $cosmosStore.connectionStatus !== "connected"
) {
  title = "Connect Wallet"
  text = "Please connect a wallet to continue using the app."
} else {
  title = ""
  text = ""
}
</script>

<section class="max-w-lg">
  {#if title}
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
  {/if}
</section>
