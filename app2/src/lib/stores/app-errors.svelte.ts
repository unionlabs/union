import { Array as A, Option, pipe } from "effect"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"

// Get all token errors from the store
const _tokenErrors = $derived(
  pipe(
    A.fromIterable(tokensStore.error),
    A.filterMap(([a, b]) => Option.all([Option.some(a), b])),
    A.map(([chainId, error]) => ({
      chainId,
      error
    }))
  )
)
export const tokenErrors = () => _tokenErrors

const _totalErrorCount = $derived((Option.isSome(chains.error) ? 1 : 0) + _tokenErrors.length)

export const totalErrorCount = () => _totalErrorCount
