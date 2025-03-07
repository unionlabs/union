import { Option } from "effect"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"

// Get all token errors from the store
const _tokenErrors = $derived(
  Array.from(tokensStore.error.entries())
    .filter(([_, error]) => Option.isSome(error))
    .map(([chainId, error]) => ({
      chainId,
      error: error.value // valid given prior filter
    }))
)
export const tokenErrors = () => _tokenErrors

const _totalErrorCount = $derived((Option.isSome(chains.error) ? 1 : 0) + _tokenErrors.length)

export const totalErrorCount = () => _totalErrorCount
