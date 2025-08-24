import { runFork, runPromise, runSync } from "$lib/runtime"
import { fetchAptosBalance, type FetchAptosBalanceError } from "$lib/services/aptos/balances"
import { fetchCosmosBalance, type FetchCosmosBalanceError } from "$lib/services/cosmos/balances"
import { fetchEvmBalance, type FetchEvmBalanceError } from "$lib/services/evm/balances"
import type { Chain, TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"
import {
  type AddressCanonicalBytes,
  AddressCosmosCanonical,
  AddressEvmCanonical,
  RawTokenBalance,
} from "@unionlabs/sdk/schema"
import {
  Array as Arr,
  type Duration,
  Effect,
  type Fiber,
  FiberMap,
  flow,
  Match,
  Option,
  pipe,
  Scope,
  Stream,
  String as Str,
} from "effect"
import { SvelteMap } from "svelte/reactivity"

const MAX_FETCH_DELAY_MS = 500

// Composite key type for the maps
export type BalanceKey = `${UniversalChainId}:${AddressCanonicalBytes}:${TokenRawDenom}`

// Helper to create the composite key
export const createKey = (
  universalChainId: UniversalChainId,
  address: AddressCanonicalBytes,
  denom: TokenRawDenom,
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
  address: AddressCanonicalBytes,
): ChainKey => `${universalChainId}:${address}`

// TODO: move into ADT; remove throw
export const denomFromChainKey = flow(Str.split(":"), Arr.last, Option.getOrThrow)

// TODO: move into ADT
export const balanceKeyFromRequest = (x: BalanceFetchRequest): BalanceKey =>
  `${x.chain.universal_chain_id}:${x.address}:${x.denom}`

// TODO: move me; or turn me into a schedule
const boundedFibonacci$: (maxDelta: number) => Stream.Stream<number> = maxDelta =>
  Stream.unfold<[number, number], number>([1, 1], ([a, b]) => {
    const next = a + b
    const delta = next - b
    const result = delta > maxDelta ? a + maxDelta : next
    return Option.some([b, [b, result]])
  })

export type BalancesStoreError =
  | FetchEvmBalanceError
  | FetchCosmosBalanceError
  | FetchAptosBalanceError

export class BalancesStore {
  data = $state(new SvelteMap<BalanceKey, RawTokenBalance>())
  errors = $state(
    new SvelteMap<
      BalanceKey,
      Option.Option<BalancesStoreError>
    >(),
  )
  chainFibers = $state(new SvelteMap<ChainKey, Fiber.RuntimeFiber<void, never>>())
  pendingRequests = $state(new SvelteMap<ChainKey, Array<BalanceFetchRequest>>())
  #scope: Scope.Scope
  #fiberMap: FiberMap.FiberMap<BalanceKey>

  constructor() {
    this.#scope = pipe(Scope.make(), runSync)
    this.#fiberMap = pipe(FiberMap.make<BalanceKey>(), Scope.extend(this.#scope), runSync)
  }

  setBalance(
    universalChainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom,
    balance: RawTokenBalance,
  ) {
    this.data.set(createKey(universalChainId, address, denom), balance)
  }

  setError(
    universalChainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom,
    error: Option.Option<BalancesStoreError>,
  ) {
    this.errors.set(createKey(universalChainId, address, denom), error)
  }

  getBalance(
    chainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom,
  ): RawTokenBalance {
    // console.log({ data: this.data })
    return this.data.get(createKey(chainId, address, denom)) ?? RawTokenBalance.make(Option.none())
  }

  getError(
    universalChainId: UniversalChainId,
    address: AddressCanonicalBytes,
    denom: TokenRawDenom,
  ): Option.Option<BalancesStoreError> {
    return this.errors.get(createKey(universalChainId, address, denom)) ?? Option.none()
  }

  interruptBalanceFetching() {
    runPromise(FiberMap.clear(this.#fiberMap))
  }

  private processBatchedBalances(
    chain: Chain,
    address: AddressCanonicalBytes,
    denoms: ReadonlyArray<TokenRawDenom>,
    interval: Duration.DurationInput,
  ) {
    this.interruptBalanceFetching()

    const balanceFetchRequestPayloads: ReadonlyArray<BalanceFetchRequest> = denoms.map(denom => ({
      chain,
      address,
      denom,
    }))

    const fetchBalance = Match.type<BalanceFetchRequest>().pipe(
      Match.when({ chain: { rpc_type: "evm" } }, ({ chain, address, denom }) =>
        fetchEvmBalance({
          chain,
          tokenAddress: denom,
          walletAddress: AddressEvmCanonical.make(address),
        })),
      Match.when(
        { chain: { rpc_type: "aptos" } },
        ({ chain, address, denom }) =>
          fetchAptosBalance({
            chain,
            tokenAddress: denom,
            walletAddress: address,
          }),
      ),
      Match.when(
        { chain: { rpc_type: "cosmos" } },
        ({ chain, address, denom }) =>
          fetchCosmosBalance({
            chain,
            tokenAddress: denom,
            walletAddress: AddressCosmosCanonical.make(address),
          }),
      ),
      Match.orElseAbsurd,
    )

    const boundedDelay = boundedFibonacci$(MAX_FETCH_DELAY_MS)
    const fetchRequest$ = pipe(
      Stream.fromIterable(balanceFetchRequestPayloads),
      Stream.zip(boundedDelay),
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
                balance,
              )
              this.setError(
                request.chain.universal_chain_id,
                request.address,
                request.denom,
                Option.none(),
              )
            }),
            /**
             * Fork and collect in `FiberMap`.
             */
            FiberMap.run(this.#fiberMap, balanceKeyFromRequest(request)),
          ),
        {
          concurrency: "unbounded",
          unordered: false,
        },
      ),
      /**
       * Re-initiate balance queries on given schedule.
       */
      Stream.runDrain,
    )

    runFork(batchProcessor)
  }

  fetchBalances(
    chain: Chain,
    address: AddressCanonicalBytes,
    denom: ReadonlyArray<TokenRawDenom> | TokenRawDenom,
    interval: Duration.DurationInput | undefined = "60 seconds",
  ) {
    this.processBatchedBalances(chain, address, Arr.ensure(denom), interval)
  }
}

export const balancesStore = new BalancesStore()
