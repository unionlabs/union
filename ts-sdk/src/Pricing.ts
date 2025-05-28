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
import { Array as A, Data, Effect, Layer, Option as O, Stream } from "effect"
import { constant, flow, hole, pipe } from "effect/Function"
import { TokenRepresentation } from "./schema/token.js"

export class PricingError extends Data.TaggedError("PricingError")<{
  message: string
  cause?: unknown
}> {}

export class Pricing extends Effect.Service<Pricing>()("Pricing", {
  // NOTE: The backing-agnostic service contract
  sync: () => ({
    of: hole<(token: TokenRepresentation) => Effect.Effect<any, any>>(),
    stream: hole<(token: TokenRepresentation) => Stream.Stream<any, any>>(),
  }),
}) {
  // TODO: Replace with FastCheck arbitraries
  static Test = Layer.sync(
    Pricing,
    constant(Pricing.make({
      of: () => Effect.void,
      stream: () => Stream.void,
    })),
  )

  static Pyth = Layer.effect(
    Pricing,
    Effect.gen(function*() {
      const { HermesClient } = yield* Effect.tryPromise({
        try: () => import("@pythnetwork/hermes-client"),
        catch: (cause) =>
          new PricingError({
            message: "Unable to import Hermes client.",
            cause,
          }),
      })

      const queryPriceFeed = yield* (Effect.cachedFunction((symbol: string) =>
        Effect.tryPromise({
          try: () =>
            client.getPriceFeeds({
              query: `${symbol}/USD`,
              assetType: "crypto",
            }),
          catch: (cause) =>
            new PricingError({
              message: `Failed to fetch pricing feed for ${symbol}.`,
              cause,
            }),
        })
      ))

      // TODO: move URL resource into dependency
      const client = new HermesClient("https://hermes.pyth.network")

      // TODO: probably make this accept variadic arguments or ensure array; alternatively
      //       this can be abstracted
      const feedIdOf = Effect.fn("feedIdOf")((token: TokenRepresentation) =>
        pipe(
          queryPriceFeed(token.symbol),
          // TODO: move into helper or extend `queryPriceFeed`
          Effect.flatMap(flow(
            A.findFirst(x =>
              // TODO: check safely
              x.attributes["base"] === token.symbol && x.attributes["quote_currency"] === "USD"
            ),
            O.map(x => x.id),
          )),
          Effect.catchTag(
            "NoSuchElementException",
            (cause) =>
              new PricingError({
                message: `Failed to capture feed ID for ${token.symbol}.`,
                cause,
              }),
          ),
        )
      )

      const getLatestPriceUpdate = Effect.fn("getLatestPriceUpdates")(
        (id: string) =>
          Effect.tryPromise({
            try: () => client.getLatestPriceUpdates([id], { parsed: true }),
            catch: (cause) =>
              new PricingError({
                message: `Failed to fetch price for feed ID ${id}`,
                cause,
              }),
          }).pipe(
            Effect.map(
              ({ parsed }) => {
                const { price: { price, expo } } = (parsed as NonNullable<typeof parsed>)[0]
                return +price * Math.pow(10, expo)
              },
            ),
          ),
      )

      return Pricing.make({
        of: Effect.fn("usdOf")(flow(
          feedIdOf,
          Effect.flatMap(getLatestPriceUpdate),
        )),
        stream: () => Stream.fail(new PricingError({ message: "not implemented" })),
      })
    }),
  )
}
