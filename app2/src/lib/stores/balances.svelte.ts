import { Effect, type Fiber, Option, Queue } from "effect"
import type { TokenRawDenom } from "$lib/schema/token"
import type { Chain, UniversalChainId } from "$lib/schema/chain"
import { RawTokenBalance } from "$lib/schema/token"
import { fetchEvmBalance, type FetchEvmBalanceError } from "$lib/services/evm/balances"
import { fetchCosmosBalance, type FetchCosmosBalanceError } from "$lib/services/cosmos/balances"
import { SvelteMap } from "svelte/reactivity"
import {
  AddressEvmCanonical,
  AddressCosmosCanonical,
  type AddressCanonicalBytes
} from "$lib/schema/address"
import { fetchAptosBalance, type FetchAptosBalanceError } from "$lib/services/aptos/balances"

// Composite key type for the maps
export type BalanceKey = `${UniversalChainId}:${AddressCanonicalBytes}:${TokenRawDenom}`

// Helper to create the composite key
export const createKey = (
  universalChainId: UniversalChainId,
  address: AddressCanonicalBytes,
  denom: TokenRawDenom
): BalanceKey => `${universalChainId}:${address}:${denom}`

// Type for a balance fetch request
type BalanceFetchRequest = {
  chain: Chain
  address: AddressCanonicalBytes
  denom: TokenRawDenom
}

// Type for chain key
type ChainKey = `${UniversalChainId}:${AddressCanonicalBytes}`

// Helper to create the chain key
const createChainKey = (
  universalChainId: UniversalChainId,
  address: AddressCanonicalBytes
): ChainKey => `${universalChainId}:${address}`

export class BalancesStore {
  data = $state(new SvelteMap<BalanceKey, RawTokenBalance>())
  errors = $state(
    new SvelteMap<
      BalanceKey,
      Option.Option<FetchEvmBalanceError | FetchCosmosBalanceError | FetchAptosBalanceError>
    >()
  )
  chainFibers = $state(new SvelteMap<ChainKey, Fiber.RuntimeFiber<void, never>>())
  pendingRequests = $state(new SvelteMap<ChainKey, Array<BalanceFetchRequest>>())

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
    error: Option.Option<FetchEvmBalanceError | FetchCosmosBalanceError | FetchAptosBalanceError>
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
  ): Option.Option<FetchEvmBalanceError | FetchCosmosBalanceError | FetchAptosBalanceError> {
    return this.errors.get(createKey(universalChainId, address, denom)) ?? Option.none()
  }

  // Process balance requests for a specific chain one at a time
  private processBatchedBalances(
    chain: Chain,
    address: AddressCanonicalBytes,
    denoms: Array<TokenRawDenom>
  ) {
    const chainKey = createChainKey(chain.universal_chain_id, address)
    const self = this

    // If there's already a query running for this chain, don't start another one
    if (this.chainFibers.has(chainKey)) {
      // Add these requests to pending
      const existing = this.pendingRequests.get(chainKey) || []
      const newRequests = denoms.map(denom => ({ chain, address, denom }))
      this.pendingRequests.set(chainKey, [...existing, ...newRequests])
      return
    }

    // Create a queue for processing balance requests
    const batchProcessor = Effect.gen(function* (_) {
      // Create a queue for balance requests
      const queue = yield* Queue.unbounded<BalanceFetchRequest>()

      // Add all denoms to the queue
      for (const denom of denoms) {
        yield* Queue.offer(queue, { chain, address, denom })
      }

      yield* Effect.forever(
        Effect.gen(function* (_) {
          // Take the next request from the queue
          const request = yield* Queue.take(queue)
          const { chain, address, denom } = request

          // Process the balance request
          yield* Effect.gen(function* (_) {
            let balance: RawTokenBalance
            if (chain.rpc_type === "evm") {
              balance = yield* fetchEvmBalance({
                chain,
                tokenAddress: denom,
                walletAddress: AddressEvmCanonical.make(address)
              })
            } else if (chain.rpc_type === "aptos") {
              balance = yield* fetchAptosBalance({
                chain,
                tokenAddress: denom,
                walletAddress: address
              })
            } else {
              balance = yield* fetchCosmosBalance({
                chain,
                tokenAddress: denom,
                walletAddress: AddressCosmosCanonical.make(address)
              })
            }

            // Update the balance
            self.setBalance(chain.universal_chain_id, address, denom, balance)
            self.setError(chain.universal_chain_id, address, denom, Option.none())
          }).pipe(
            Effect.catchAll(error => {
              // Update the error
              self.setError(chain.universal_chain_id, address, denom, Option.some(error))
              return Effect.succeed(undefined)
            })
          )
        })
      )
    }).pipe(
      Effect.catchAll(error => {
        Effect.logError("error processing balance batch:", error)
        return Effect.succeed(undefined)
      }),
      Effect.ensuring(
        Effect.sync(() => {
          // Check if there are pending requests for this chain
          const pending = self.pendingRequests.get(chainKey) || []
          self.pendingRequests.delete(chainKey)
          self.chainFibers.delete(chainKey)

          // If there are pending requests, process them
          if (pending.length > 0) {
            const pendingDenoms = pending.map(req => req.denom)
            self.processBatchedBalances(chain, address, pendingDenoms)
          }
        })
      )
    )

    // Run the batch processor
    const fiber = Effect.runFork(batchProcessor)
    this.chainFibers.set(chainKey, fiber)
  }

  fetchBalance(chain: Chain, address: AddressCanonicalBytes, denom: TokenRawDenom) {
    this.processBatchedBalances(chain, address, [denom])
  }

  fetchBalances(chain: Chain, address: AddressCanonicalBytes, denoms: Array<TokenRawDenom>) {
    if (denoms.length === 0) return
    this.processBatchedBalances(chain, address, denoms)
  }
}

export const balancesStore = new BalancesStore()
