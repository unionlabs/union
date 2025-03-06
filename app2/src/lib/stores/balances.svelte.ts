import { Effect, type Fiber, Option } from "effect"
import { TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { UniversalChainId } from "$lib/schema/chain"
import { RawTokenBalance } from "$lib/schema/token"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { SvelteMap } from "svelte/reactivity"

// Composite key type for the maps
type BalanceKey = `${UniversalChainId}:${TokenRawDenom}`

// Helper to create the composite key
const createKey = (universalChainId: UniversalChainId, denom: TokenRawDenom): BalanceKey =>
  `${universalChainId}:${denom}`

class BalancesStore {
  data = $state(new SvelteMap<BalanceKey, RawTokenBalance>())
  errors = $state(new SvelteMap<BalanceKey, Option.Option<FetchDecodeGraphqlError>>())
  fibers = $state(new SvelteMap<BalanceKey, Fiber.RuntimeFiber<number, never>>())

  setBalance(
    universalChainId: typeof UniversalChainId.Type,
    denom: typeof TokenRawDenom.Type,
    balance: typeof RawTokenBalance.Type
  ) {
    this.data.set(createKey(universalChainId, denom), balance)
  }

  setError(
    universalChainId: UniversalChainId,
    denom: TokenRawDenom,
    error: Option.Option<FetchDecodeGraphqlError>
  ) {
    this.errors.set(createKey(universalChainId, denom), error)
  }

  getBalance(chainId: UniversalChainId, denom: TokenRawDenom): typeof RawTokenBalance.Type {
    return this.data.get(createKey(chainId, denom)) ?? RawTokenBalance.make(Option.none())
  }

  getError(
    universalChainId: UniversalChainId,
    denom: TokenRawDenom
  ): Option.Option<FetchDecodeGraphqlError> {
    return this.errors.get(createKey(universalChainId, denom)) ?? Option.none()
  }

  fetchBalance(
    universalChainId: UniversalChainId,
    denom: TokenRawDenom,
    effect: Effect.Effect<never, never, number>
  ) {
    const key = createKey(universalChainId, denom)

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
