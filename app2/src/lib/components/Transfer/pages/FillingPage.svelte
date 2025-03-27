<script lang="ts">
import ChainAsset from "$lib/components/Transfer/ChainAsset/index.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import Button from "$lib/components/ui/Button.svelte"
import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"

type Props = {
  onContinue: () => void
  actionButtonText: string
}

const { onContinue, actionButtonText }: Props = $props()

</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  <div class="flex flex-col gap-4">
    <ChainAsset type="source"/>
    <ChainAsset type="destination"/>
    <Amount type="source"/>
  </div>

  <div class="flex flex-col items-end">
    <div class="flex items-center mr-5 text-zinc-400">
      {#if transfer.args.receiver && transfer.args.destinationChain}
        <p class="text-xs mb-2"><AddressComponent truncate address={transfer.raw.receiver} chain={transfer.args.destinationChain}/></p>
      {:else}
        <p class="text-xs mb-2"> No receiver</p>
      {/if}
      <AngleArrowIcon class="rotate-270"/>
    </div>
    <div class="w-full items-end flex gap-2">
      <Button
        class="flex-1"
        variant="primary"
        onclick={onContinue}
        disabled={transfer.validation._tag !== "Success"}
      >
        {actionButtonText}
      </Button>
      <Receiver/>
    </div>
  </div>
</div>
