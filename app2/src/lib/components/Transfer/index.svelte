<script lang="ts">
import Chain from "$lib/components/Transfer/Chain.svelte"
import Card from "$lib/components/ui/Card.svelte"
import {
  hasFailedExit as hasCosmosFailedExit,
  isComplete as isCosmosComplete
} from "$lib/services/transfer-cosmos"
import {
  hasFailedExit as hasEvmFailedExit,
  isComplete as isEvmComplete
} from "$lib/services/transfer-ucs03-evm"
import Button from "$lib/components/ui/Button.svelte"
import Assets from "$lib/components/Transfer/Assets.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import ShowData from "$lib/components/Transfer/ShowData.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option } from "effect"

$effect(() => {
  transfer.getQuoteToken()
  transfer.getWethQuoteToken()
})

function hasFailedExit(state) {
  if (!state) return false

  return Option.isSome(transfer.sourceChain) && transfer.sourceChain.value.rpc_type === "evm"
    ? hasEvmFailedExit(state)
    : hasCosmosFailedExit(state)
}

function isComplete(state) {
  if (!state) return false

  return Option.isSome(transfer.sourceChain) && transfer.sourceChain.value.rpc_type === "evm"
    ? isEvmComplete(state)
    : isCosmosComplete(state)
}

let isButtonEnabled = $derived(
  transfer.state &&
    (transfer.state._tag === "Filling" ||
      hasFailedExit(transfer.state) ||
      isComplete(transfer.state))
)
</script>

<Card class="max-w-md relative flex flex-col gap-2">
  <Chain type="source"/>
  <Chain type="destination"/>
  <Assets/>
  <Amount/>
  <Receiver/>

  <!-- For testing -->
  <ShowData/>
  <!-- For testing -->

  <Button
          class="mt-2"
          variant="primary"
          onclick={transfer.submit}
          disabled={!isButtonEnabled}
  >
    {#if !transfer.state}
      Select
    {:else if transfer.state._tag !== "Filling" && !hasFailedExit(transfer.state) && !isComplete(transfer.state)}
      Submitting...
    {:else if hasFailedExit(transfer.state)}
      Retry
    {:else}
      Submit
    {/if}
  </Button>
</Card>
{#if transfer.state}
  {JSON.stringify(transfer.state, null, 2)}
{/if}