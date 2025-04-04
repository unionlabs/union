import {
  Array as A,
  String as ST,
  Effect,
  Fiber,
  FiberMap,
  type FiberStatus,
  Match,
  Option,
  pipe,
  Schedule,
  Stream,
  flow,
  Scope
} from "effect"
import type { Chain, TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"
import {
  type AddressCanonicalBytes,
  AddressCosmosCanonical,
  AddressEvmCanonical,
  RawTokenBalance
} from "@unionlabs/sdk/schema"
import { fetchEvmBalance, type FetchEvmBalanceError } from "$lib/services/evm/balances"
import { fetchCosmosBalance, type FetchCosmosBalanceError } from "$lib/services/cosmos/balances"
import { SvelteMap } from "svelte/reactivity"
import { fetchAptosBalance, type FetchAptosBalanceError } from "$lib/services/aptos/balances"
import type { Tags } from "effect/Types"

// TODO: move me
const fibonacci$: Stream.Stream<number> = Stream.unfold<[number, number], number>(
  [1, 1],
  ([a, b]) => Option.some([b, [b, a + b]])
)

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

// TODO: move into ADT
export const denomFromChainKey = flow(ST.split(":"), A.last, Option.getOrThrow)

// TODO: move into ADT
export const balanceKeyFromRequest = (x: BalanceFetchRequest): BalanceKey =>
  `${x.chain.universal_chain_id}:${x.address}:${x.denom}`

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
  scope = Effect.runSync(Scope.make())
  fiberMap = FiberMap.make<BalanceKey>().pipe(Scope.extend(this.scope), Effect.runSync)

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

  get fiberMapStatuses$(): Stream.Stream<[BalanceKey, Tags<FiberStatus.FiberStatus>]> {
    return pipe(
      Stream.fromIterable(this.fiberMap),
      Stream.mapEffect(
        ([key, fiber]) =>
          pipe(
            Fiber.status(fiber),
            Effect.map(x => x._tag),
            Effect.map(status => [key, status])
          ),
        {
          concurrency: "unbounded",
          unordered: true
        }
      )
    )
  }

  // TODO: naming
  stopFetching() {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this
    Effect.gen(function* () {
      yield* FiberMap.clear(self.fiberMap)
      const batchFiber = Option.getOrElse(self.aFiber, () => Fiber.void)
      yield* Fiber.interrupt(batchFiber)
    }).pipe(Effect.runPromise)
  }

  private processBatchedBalances(
    chain: Chain,
    address: AddressCanonicalBytes,
    denoms: ReadonlyArray<TokenRawDenom>
  ) {
    const balanceFetchRequestPayloads: ReadonlyArray<BalanceFetchRequest> = denoms.map(denom => ({
      chain,
      address,
      denom
    }))

    const fetchBalance = Match.type<BalanceFetchRequest>().pipe(
      Match.when({ chain: { rpc_type: "evm" } }, ({ chain, address, denom }) =>
        fetchEvmBalance({
          chain,
          tokenAddress: denom,
          walletAddress: AddressEvmCanonical.make(address)
        })
      ),
      Match.when({ chain: { rpc_type: "aptos" } }, ({ chain, address, denom }) =>
        fetchAptosBalance({
          chain,
          tokenAddress: denom,
          walletAddress: address
        })
      ),
      Match.when({ chain: { rpc_type: "cosmos" } }, ({ chain, address, denom }) =>
        fetchCosmosBalance({
          chain,
          tokenAddress: denom,
          walletAddress: AddressCosmosCanonical.make(address)
        })
      ),
      Match.orElseAbsurd
    )

    const fetchRequest$ = pipe(
      Stream.fromIterable(balanceFetchRequestPayloads),
      Stream.zip(fibonacci$)
    )

    const batchProcessor = fetchRequest$.pipe(
      /**
       * Preload all requests to be executed after given delay.
       *
       * This ensures that the `FiberMap` is populated fully and then reduced
       * to empty.
       */
      Stream.mapEffect(
        ([request, delay]) =>
          pipe(
            /**
             * Delay fetching with zipped millisecond stream.
             */
            fetchBalance(request),
            Effect.delay(delay),
            /**
             * Set data and error runes.
             */
            Effect.tap(balance => {
              this.setBalance(
                request.chain.universal_chain_id,
                request.address,
                request.denom,
                balance
              )
              this.setError(
                request.chain.universal_chain_id,
                request.address,
                request.denom,
                Option.none()
              )
            }),
            /**
             * Fork and collect in `FiberMap`.
             */
            FiberMap.run(this.fiberMap, balanceKeyFromRequest(request))
          ),
        {
          concurrency: "unbounded",
          unordered: false
        }
      ),
      /**
       * Re-initiate balance queries on given schedule.
       */
      Stream.repeat(Schedule.spaced("60 seconds")),
      Stream.runDrain
    )

    // TODO: cleanup fiber (?)
    Effect.runFork(batchProcessor)
  }

  fetchBalances(
    chain: Chain,
    address: AddressCanonicalBytes,
    denom: ReadonlyArray<TokenRawDenom> | TokenRawDenom
  ) {
    this.processBatchedBalances(chain, address, A.ensure(denom))
  }
}

export const balancesStore = new BalancesStore()
