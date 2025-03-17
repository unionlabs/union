<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import ShowData from "$lib/components/Transfer/ShowData.svelte"
import { transfer, type TransferStateUnion } from "$lib/components/Transfer/transfer.svelte.ts"
import {
  hasFailedExit as hasCosmosFailedExit,
  isComplete as isCosmosComplete
} from "$lib/services/transfer-ucs03-cosmos"
import {
  hasFailedExit as hasEvmFailedExit,
  isComplete as isEvmComplete
} from "$lib/services/transfer-ucs03-evm"
import {
  hasFailedExit as hasAptosFailedExit,
  isComplete as isAptosComplete
} from "$lib/services/transfer-ucs03-aptos"
import ChainAsset from "$lib/components/Transfer/ChainAsset/index.svelte"

$effect(() => {
  transfer.getQuoteToken()
  transfer.getWethQuoteToken()
})

// Simplified status checker using the enum
function getStatus(
  state: TransferStateUnion
): "empty" | "filling" | "processing" | "failed" | "complete" {
  switch (state._tag) {
    case "Empty":
      return "empty"
    case "EVM": {
      if (state.state._tag === "Filling") return "filling"
      if (hasEvmFailedExit(state.state)) return "failed"
      if (isEvmComplete(state.state)) return "complete"
      return "processing"
    }
    case "Cosmos": {
      if (state.state._tag === "Filling") return "filling"
      if (hasCosmosFailedExit(state.state)) return "failed"
      if (isCosmosComplete(state.state)) return "complete"
      return "processing"
    }
    case "Aptos": {
      if (state.state._tag === "Filling") return "filling"
      if (hasAptosFailedExit(state.state)) return "failed"
      if (isAptosComplete(state.state)) return "complete"
      return "processing"
    }
    default:
      return "empty"
  }
}

function getError(state: TransferStateUnion): string | null {
  switch (state._tag) {
    case "Empty":
      return null
    case "EVM":
    case "Aptos":
    case "Cosmos": {
      const innerState = state.state
      if (innerState._tag === "Filling") return null
      if (
        innerState.state._tag === "Complete" &&
        innerState.state.exit._tag === "Failure" &&
        "cause" in innerState.state.exit
      ) {
        const cause = innerState.state.exit.cause
        return typeof cause === "object" && cause && "cause" in cause
          ? String(cause.cause)
          : String(cause)
      }
      return null
    }
    default:
      return null
  }
}

// Simplified step name extractor
function getStepName(state: TransferStateUnion): string | null {
  switch (state._tag) {
    case "Empty":
      return null
    case "EVM":
      return state.state._tag
    case "Aptos":
      return state.state._tag
    case "Cosmos":
      return state.state._tag
    default:
      return null
  }
}

let isButtonEnabled = $derived(
  getStatus(transfer.state) === "filling" ||
    getStatus(transfer.state) === "failed" ||
    getStatus(transfer.state) === "complete"
)

let buttonText = $derived(
  {
    empty: "Select",
    filling: "Submit",
    processing: "Submitting...",
    failed: "Retry",
    complete: "Submit"
  }[getStatus(transfer.state)]
)
</script>

<Card class="max-w-md relative flex flex-col gap-2">
  <ChainAsset type="source" />
  <ChainAsset type="destination" />
  <Amount />
  <Receiver />
  <ShowData />
  <Button
          class="mt-2"
          variant="primary"
          onclick={transfer.submit}
          disabled={!isButtonEnabled}
  >
    {buttonText}
  </Button>
</Card>

{#if transfer.state._tag !== "Empty"}
  {#if getStatus(transfer.state) === "filling"}
    <div>Select assets and amounts to begin transfer.</div>
  {:else if getStatus(transfer.state) === "failed"}
    <div style="color: red;">Error: {getError(transfer.state) ?? "Unknown error"}</div>
  {:else if getStatus(transfer.state) === "processing"}
    <div>Processing {getStepName(transfer.state) ?? "step"}...</div>
  {:else if getStatus(transfer.state) === "complete"}
    <div style="color: green;">Transfer completed successfully!</div>
  {/if}
  <pre>{JSON.stringify(transfer.state, null, 2)}</pre>
{/if}