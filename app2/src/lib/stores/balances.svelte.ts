import { Effect, type Fiber, Option } from "effect"
import type { TokenRawDenom } from "$lib/schema/token"
import type { UniversalChainId } from "$lib/schema/chain"
import type { RawTokenBalance } from "$lib/schema/token"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { SvelteMap } from "svelte/reactivity"

// Composite key type for the maps
type BalanceKey = `${typeof UniversalChainId.Type}:${typeof TokenRawDenom.Type}`

// Helper to create the composite key
const createKey = (
  chainId: typeof UniversalChainId.Type,
  denom: typeof TokenRawDenom.Type
): BalanceKey => `${chainId}:${denom}`

class BalancesStore {
  data = $state(new SvelteMap<BalanceKey, typeof RawTokenBalance.Type>())
  errors = $state(new SvelteMap<BalanceKey, Option.Option<FetchDecodeGraphqlError>>())
  fibers = $state(new SvelteMap<BalanceKey, Fiber.RuntimeFiber<number, never>>())

  setBalance(
    chainId: typeof UniversalChainId.Type,
    denom: typeof TokenRawDenom.Type,
    balance: typeof RawTokenBalance.Type
  ) {
    this.data.set(createKey(chainId, denom), balance)
  }

  setError(
    chainId: typeof UniversalChainId.Type,
    denom: typeof TokenRawDenom.Type,
    error: Option.Option<FetchDecodeGraphqlError>
  ) {
    this.errors.set(createKey(chainId, denom), error)
  }

  getBalance(
    chainId: typeof UniversalChainId.Type,
    denom: typeof TokenRawDenom.Type
  ): typeof RawTokenBalance.Type {
    return this.data.get(createKey(chainId, denom)) ?? Option.none()
  }

  getError(
    chainId: typeof UniversalChainId.Type,
    denom: typeof TokenRawDenom.Type
  ): Option.Option<FetchDecodeGraphqlError> {
    return this.errors.get(createKey(chainId, denom)) ?? Option.none()
  }

  fetchBalance(
    chainId: typeof UniversalChainId.Type,
    denom: typeof TokenRawDenom.Type,
    effect: Effect.Effect<never, never, number>
  ) {
    const key = createKey(chainId, denom)

    // If there's already a query running for this combination, don't start another one
    if (this.fibers.has(key)) {
      return
    }

    // Start new query and store its fiber
    const fiber = Effect.runFork(effect)
    this.fibers.set(key, fiber)
  }
}

export const balancesStore = new BalancesStore()
