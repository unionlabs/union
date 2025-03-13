<script lang="ts">
import Chain from "$lib/components/Transfer/Chain.svelte"
import Card from "$lib/components/ui/Card.svelte"
import {
  hasFailedExit as hasCosmosFailedExit,
  isComplete as isCosmosComplete
} from "$lib/services/transfer-ucs03-cosmos"
import {
  hasFailedExit as hasEvmFailedExit,
  isComplete as isEvmComplete
} from "$lib/services/transfer-ucs03-evm"
import Button from "$lib/components/ui/Button.svelte"
import Assets from "$lib/components/Transfer/Assets.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import ShowData from "$lib/components/Transfer/ShowData.svelte"
import {
  transfer,
  type TransferStateUnion as TransferState
} from "$lib/components/Transfer/transfer.svelte.ts"

$effect(() => {
  transfer.getQuoteToken()
  transfer.getWethQuoteToken()
})

function hasFailedExit(state: TransferState) {
  if (state._tag === "Empty") return false
  if (state._tag === "EVM") return hasEvmFailedExit(state.state)
  if (state._tag === "Cosmos") return hasCosmosFailedExit(state.state)
  return false
}

function isComplete(state: TransferState) {
  if (state._tag === "Empty") return false
  if (state._tag === "EVM") return isEvmComplete(state.state)
  if (state._tag === "Cosmos") return isCosmosComplete(state.state)
  return false
}

function getInnerTag(state: TransferState) {
  if (state._tag === "Empty") return null
  if (state._tag === "EVM" || state._tag === "Cosmos") return state.state._tag
  return null
}

let isButtonEnabled = $derived(
  transfer.state._tag !== "Empty" &&
    (getInnerTag(transfer.state) === "Filling" ||
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
    {#if transfer.state._tag === "Empty"}
      Select
    {:else if getInnerTag(transfer.state) !== "Filling" && !hasFailedExit(transfer.state) && !isComplete(transfer.state)}
      Submitting...
    {:else if hasFailedExit(transfer.state)}
      Retry
    {:else}
      Submit
    {/if}
  </Button>
</Card>
{#if transfer.state._tag !== "Empty"}
  {JSON.stringify(transfer.state, null, 2)}
{/if}