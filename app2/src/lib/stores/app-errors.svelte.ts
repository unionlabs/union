import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { Array as A, Option, pipe } from "effect"
import { channels } from "./channels.svelte"

// Get all token errors from the store
const _tokenErrors = $derived(
  pipe(
    A.fromIterable(tokensStore.error),
    A.filterMap(([a, b]) => Option.all([Option.some(a), b])),
    A.map(([chainId, error]) => ({
      chainId,
      error,
    })),
  ),
)
export const tokenErrors = () => _tokenErrors

const _totalErrorCount = $derived(pipe(
  A.fromIterable([
    chains.error,
    channels.error,
  ]),
  A.getSomes,
  A.length,
))

export const totalErrorCount = () => _totalErrorCount
