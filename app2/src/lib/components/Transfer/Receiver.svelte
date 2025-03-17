<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option } from "effect"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { uiStore } from "$lib/stores/ui.svelte.ts"

let destinationChain = $derived(
  Option.isSome(transfer.destinationChain) ? Option.getOrNull(transfer.destinationChain) : null
)
let hasWalletAddress = $derived(
  destinationChain && Option.isSome(wallets.getAddressForChain(destinationChain))
)
</script>

<Input
        type="text"
        id="receiver"
        label="receiver"
        required={true}
        disabled={!transfer.raw.destination}
        autocorrect="off"
        spellcheck="false"
        autocomplete="off"
        data-field="receiver"
        placeholder="Enter destination address"
        value={transfer.raw.receiver}
        oninput={event => transfer.raw.updateField('receiver', event)}
        class="text-center"
/>

{#if Option.isSome(transfer.destinationChain)}
  <div class="flex w-full justify-end">
    <button
            onclick={() => {
              if (transfer.raw.receiver) {
                transfer.raw.updateField("receiver", "");
              } else if (hasWalletAddress && destinationChain) {
                const address = wallets.getAddressForChain(destinationChain);
                transfer.raw.updateField("receiver", Option.getOrNull(address));
              } else {
                uiStore.walletModalOpen = true;
              }
            }}
            class="text-xs cursor-pointer hover:underline"
    >
      {#if transfer.raw.receiver}
        RESET
      {:else if hasWalletAddress}
        USE CONNECTED WALLET
      {:else}
        CONNECT <span class="uppercase">{transfer.destinationChain.value.rpc_type} WALLET</span>
      {/if}
    </button>
  </div>
{/if}