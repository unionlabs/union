<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
import { sepoliaStore } from "$lib/wallet/evm/config.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
    import AlphaNotice from "$lib/components/alpha-notice.svelte";
    import { summarizeString } from "$lib/utilities/format";

</script>

<main class="flex flex-col items-center size-full p-4 mt-16">
  <Card.Root class="max-w-lg">
    <Card.Header>
      <Card.Title>Welcome to Union</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col gap-2">
      <p>Connect an <b>EVM</b> and <b>Cosmos</b> wallet to begin bridging.</p>

      <div>
        {#if $sepoliaStore.address }
          ✅ EVM wallet <span class="font-mono">{summarizeString($sepoliaStore.address, 6)}</span> connected
        {:else}
          Connect EVM wallet
        {/if}
      </div>

      <div>
        {#if $cosmosStore.address }
          ✅Cosmos wallet connected
        {:else}
          Connect cosmos wallet
        {/if}
      </div>
    </Card.Content>
  </Card.Root>
</main>
