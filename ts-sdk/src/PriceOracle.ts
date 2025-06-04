/**
 * Determine pricing of given token representations.
 *
 * TODO:
 * - Use [ExecutionPlan](https://effect.website/blog/releases/effect/316/#executionplan-module)
 *   to fallback to different pricing sources.
 * - Can `Pricing.Default` layer expose the execution plan?
 * - Make mainnet vs testnet distinction.
 * - (optional) Match selection of potential services by chain if source is chain-specific.
 * - (optional) Allow for choosing localized currency such as not to hardcode USD.
 */
import {
  Array as A,
  Data,
  Effect,
  Layer,
  Option as O,
  Record as R,
  Schedule,
  Schema as S,
  Stream,
} from "effect"
import { flow, hole, pipe } from "effect/Function"
import { UniversalChainId } from "./schema/chain.js"

export class PriceError extends Data.TaggedError("@unionlabs/sdk/PriceOracle/PriceError")<{
  message: string
  cause?: unknown
}> {}

export const PriceSource = S.Struct({
  url: S.URL,
})
export type PriceSource = typeof PriceSource.Type

export const PriceResult = S.Struct({
  price: S.Positive.pipe(
    S.annotations({
      arbitrary: () => (fc) => fc.float({ min: Math.fround(0.1), max: Math.fround(200) }),
    }),
  ),
  source: PriceSource,
})
export type PriceResult = typeof PriceResult.Type

export class PriceOracle extends Effect.Service<PriceOracle>()("@unionlabs/sdk/PriceOracle", {
  // NOTE: The backing-agnostic service contract
  sync: () => ({
    // TODO: universal chain id static mapping
    of: hole<(id: UniversalChainId) => Effect.Effect<PriceResult, PriceError>>(),
    stream: hole<(id: UniversalChainId) => Stream.Stream<PriceResult, PriceError>>(),
    ratio: hole<
      (
        from: UniversalChainId,
        to: UniversalChainId,
      ) => Effect.Effect<{
        ratio: number
        source: PriceSource
        destination: PriceSource
      }, PriceError>
    >(),
  }),
}) {
  static Test = Layer.effect(
    PriceOracle,
    Effect.gen(function*() {
      const fc = yield* Effect.tryPromise(() => import("effect/FastCheck"))
      const Arbitrary = yield* Effect.tryPromise(() => import("effect/Arbitrary"))
      const ArbitraryPriceResult = Arbitrary.make(PriceResult)

      const of: PriceOracle["of"] = () =>
        pipe(
          fc.sample(ArbitraryPriceResult, 1)[0],
          Effect.succeed,
        )

      const stream: PriceOracle["stream"] = () =>
        pipe(
          fc.infiniteStream(ArbitraryPriceResult),
          (arb) => fc.sample(arb, 1)[0],
          Stream.fromIterable,
          Stream.schedule(Schedule.spaced("3 seconds")),
        )

      const ratio: PriceOracle["ratio"] = (a, b) =>
        pipe(
          Effect.all([of(a), of(b)], { concurrency: "unbounded" }),
          Effect.map(([a, b]) => ({
            ratio: a.price / b.price,
            source: a.source,
            destination: a.source,
          })),
        )

      return PriceOracle.make({
        of,
        stream,
        ratio,
      })
    }),
  )

  static Pyth = Layer.effect(
    PriceOracle,
    Effect.gen(function*() {
      // XXX: source from chain info?
      const map: Record<UniversalChainId, string> = {
        [UniversalChainId.make("ethereum.11155111")]: "WETH",
        [UniversalChainId.make("ethereum.1")]: "ETH",
        [UniversalChainId.make("ethereum.17000")]: "ETH",
        [UniversalChainId.make("babylon.bbn-1")]: "BABY",
        [UniversalChainId.make("babylon.bbn-test-5")]: "BABY",
      }

      const symbolFromId = Effect.fn("symbolFromId")(
        (id: UniversalChainId) =>
          pipe(
            R.get(map, id),
            Effect.mapError((cause) =>
              new PriceError({
                message: `No price ID mapping for ${id}`,
                cause,
              })
            ),
          ),
      )

      const { HermesClient } = yield* Effect.tryPromise({
        try: () => import("@pythnetwork/hermes-client"),
        catch: (cause) =>
          new PriceError({
            message: "Unable to import Hermes client.",
            cause,
          }),
      })

      const queryPriceFeed = yield* (Effect.cachedFunction((symbol: string) =>
        pipe(
          Effect.tryPromise({
            try: () =>
              client.getPriceFeeds({
                query: `${symbol}/USD`,
                assetType: "crypto",
              }),

            catch: (cause) =>
              new PriceError({
                message: `Failed to fetch pricing feed for ${symbol}.`,
                cause,
              }),
          }),
        )
      ))

      // TODO: move URL resource into dependency
      const client = new HermesClient("https://hermes.pyth.network")

      // TODO: probably make this accept variadic arguments or ensure array; alternatively
      //       this can be abstracted or implement concurrency/batching
      const feedIdOf = yield* Effect.cachedFunction(
        Effect.fn("feedIdOf")((symbol: string) =>
          pipe(
            queryPriceFeed(symbol),
            // TODO: move into helper or extend `queryPriceFeed`
            Effect.flatMap(flow(
              A.findFirst(x =>
                // TODO: check safely
                x.attributes["base"] === symbol && x.attributes["quote_currency"] === "USD"
              ),
              O.map(({ id, attributes }) => ({
                id,
                url: `https://www.pyth.network/price-feeds/${
                  attributes["asset_type"].toLowerCase()
                }-${attributes["base"].toLowerCase()}-${
                  attributes["quote_currency"].toLowerCase()
                }`,
              })),
            )),
            Effect.catchTag(
              "NoSuchElementException",
              (cause) =>
                new PriceError({
                  message: `Failed to capture feed ID for ${symbol}.`,
                  cause,
                }),
            ),
          )
        ),
      )

      const getLatestPriceUpdate = Effect.fn("getLatestPriceUpdates")(
        (id: string) =>
          pipe(
            Effect.tryPromise({
              try: () => client.getLatestPriceUpdates([id], { parsed: true }),
              catch: (cause) =>
                new PriceError({
                  message: `Failed to fetch price for feed ID ${id}`,
                  cause,
                }),
            }),
            Effect.map(
              ({ parsed }) => {
                const { price: { price, expo } } = (parsed as NonNullable<typeof parsed>)[0]
                return +price * Math.pow(10, expo)
              },
            ),
          ),
      )

      return PriceOracle.make({
        of: Effect.fn("of")(flow(
          symbolFromId,
          Effect.flatMap(feedIdOf),
          Effect.flatMap(({ id, url }) =>
            pipe(
              getLatestPriceUpdate(id),
              Effect.map((price) =>
                PriceResult.make({
                  price: price,
                  source: PriceSource.make({
                    url: new URL(url),
                  }),
                })
              ),
            )
          ),
        )),
        stream: () => Stream.fail(new PriceError({ message: "not implemented" })),
        ratio: () => Effect.fail(new PriceError({ message: "not implemented" })),
      })
    }),
  )
}
