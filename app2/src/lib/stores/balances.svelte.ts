import { Effect, type Fiber, Option } from "effect"
import type { TokenRawDenom } from "$lib/schema/token"
import type { Chain, UniversalChainId } from "$lib/schema/chain"
import { RawTokenBalance } from "$lib/schema/token"
import { createBalanceQuery, type FetchBalanceError } from "$lib/services/evm/balances"
import { SvelteMap } from "svelte/reactivity"
import { AddressEvmCanonical, type AddressCanonicalBytes } from "$lib/schema/address"

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
  errors = $state(new SvelteMap<BalanceKey, Option.Option<FetchBalanceError>>())
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
    error: Option.Option<FetchBalanceError>
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
  ): Option.Option<FetchBalanceError> {
    return this.errors.get(createKey(universalChainId, address, denom)) ?? Option.none()
  }

  fetchBalance(chain: Chain, address: AddressCanonicalBytes, denom: TokenRawDenom) {
    const key = createKey(chain.universal_chain_id, address, denom)

    // If there's already a query running for this combination, don't start another one
    if (this.fibers.has(key)) {
      return
    }

    if (chain.rpc_type === "evm") {
      const query = createBalanceQuery({
        chain,
        tokenAddress: denom,
        walletAddress: AddressEvmCanonical.make(address),
        refetchInterval: "1 minute",
        writeData: balance => this.setBalance(chain.universal_chain_id, address, denom, balance),
        writeError: error => this.setError(chain.universal_chain_id, address, denom, error)
      })

      const fiber = Effect.runFork(query)
      this.fibers.set(key, fiber)
    }
  }
}

export const balancesStore = new BalancesStore()
