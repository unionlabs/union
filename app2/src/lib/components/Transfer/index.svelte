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
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { Effect, Fiber, Option } from "effect"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"

$effect(() => {
  transfer.getQuoteToken()
  transfer.getWethQuoteToken()
 
})

$effect(() => {
  if (Option.isSome(transfer.sourceChain)) {
    tokensStore.fetchTokens(transfer.sourceChain.value.universal_chain_id)
  }
})

// Keep track of current fiber and chain
let currentFetchFiber = $state<Fiber.RuntimeFiber<void, never> | null>(null)
let currentChainId = $state<string | null>(null)
let lastFetchKey = $state("")

$effect(() => {
  // Get the current source chain
  const sourceChain = Option.getOrNull(transfer.sourceChain)

  // If the chain changed, interrupt the existing fiber
  if (sourceChain?.universal_chain_id !== currentChainId) {
    if (currentFetchFiber) {
      console.log(`Interrupting fiber for chain: ${currentChainId}`)
      Effect.runPromise(Fiber.interrupt(currentFetchFiber))
      currentFetchFiber = null
    }

    // Update the current chain ID
    currentChainId = sourceChain?.universal_chain_id || null

    // Reset the lastFetchKey when changing chains
    lastFetchKey = ""
  }

  // Exit if no source chain
  if (!sourceChain) return

  // Check for wallet address
  const addressOption = wallets.getAddressForChain(sourceChain)
  if (!Option.isSome(addressOption)) return

  const address = addressOption.value

  // Get tokens data
  const tokensOption = tokensStore.data.get(sourceChain.universal_chain_id) ?? Option.none()

  // Only proceed if we have tokens
  if (!Option.isSome(tokensOption)) return

  // Create a unique key for this combination of chain + address
  const fetchKey = `${sourceChain.universal_chain_id}:${address}`

  // Check if we need to fetch new balances
  if (fetchKey !== lastFetchKey) {
    const tokens = tokensOption.value
    const denoms = tokens.map(token => token.denom)

    // Get the chainKey to access the fiber from balancesStore
    const chainKey = `${sourceChain.universal_chain_id}:${address}` as const

    // Fetch balances for all tokens
    balancesStore.fetchBalances(sourceChain, address, denoms)

    // Store the new fiber reference
    currentFetchFiber = balancesStore.chainFibers.get(chainKey) || null

    // Update the last fetch key
    lastFetchKey = fetchKey
  }
})
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
  <ChainAsset type="source"/>
  <ChainAsset type="destination"/>
  <Amount/>
  <Receiver/>
  <ShowData/>
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
  {:else if getStatus(transfer.state) === "processing"}
    <div>Processing {getStepName(transfer.state) ?? "step"}...</div>
  {:else if getStatus(transfer.state) === "complete"}
    <div style="color: green;">Transfer completed successfully!</div>
  {/if}
  <pre>{JSON.stringify(transfer.state, null, 2)}</pre>
{/if}
