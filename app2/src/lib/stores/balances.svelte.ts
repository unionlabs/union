import { Effect, type Fiber, Option } from "effect"
import type { TokenRawDenom } from "$lib/schema/token"
import type { UniversalChainId } from "$lib/schema/chain"
import { RawTokenBalance } from "$lib/schema/token"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { SvelteMap } from "svelte/reactivity"
import type { AddressCanonicalBytes } from "$lib/schema/address"

// Composite key type for the maps
type BalanceKey = `${UniversalChainId}:${AddressCanonicalBytes}:${TokenRawDenom}`

// Helper to create the composite key
const createKey = (
  universalChainId: UniversalChainId,
  address: AddressCanonicalBytes,
  denom: TokenRawDenom
): BalanceKey => `${universalChainId}:${address}:${denom}`

class BalancesStore {
  data = $state(new SvelteMap<BalanceKey, RawTokenBalance>())
  errors = $state(new SvelteMap<BalanceKey, Option.Option<FetchDecodeGraphqlError>>())
  fibers = $state(new SvelteMap<BalanceKey, Fiber.RuntimeFiber<number, never>>())

  setBalance(
    universalChainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom,
    balance: RawTokenBalance
  ) {
    this.data.set(createKey(universalChainId, address, denom), balance)
  }

  setError(
    universalChainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom,
    error: Option.Option<FetchDecodeGraphqlError>
  ) {
    this.errors.set(createKey(universalChainId, address, denom), error)
  }

  getBalance(
    chainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom
  ): RawTokenBalance {
    return this.data.get(createKey(chainId, address, denom)) ?? RawTokenBalance.make(Option.none())
  }

  getError(
    universalChainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom
  ): Option.Option<FetchDecodeGraphqlError> {
    return this.errors.get(createKey(universalChainId, address, denom)) ?? Option.none()
  }

  fetchBalance(
    universalChainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom,
    effect: Effect.Effect<never, never, number>
  ) {
    const key = createKey(universalChainId, address, denom)

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
