/**
 * This module sources {@link Token} data.
 *
 * @since 2.0.0
 */
import { Data, Effect, Hash, hole, Layer, pipe } from "effect"
import { Chain } from "./schema/chain.js"
import { Token } from "./schema/token.js"

/**
 * @category errors
 * @since 2.0.0
 */
export class TokenRegistryError
  extends Data.TaggedError("@unionlabs/sdk/TokenRegistry/TokenRegistryError")<{
    message: string
    cause?: unknown
  }>
{}

/**
 * @category services
 * @since 2.0.0
 */
export class TokenRegistry extends Effect.Service<TokenRegistry>()("@unionlabs/sdk/TokenRegistry", {
  sync: () => ({
    pickQuote: hole<(source: Chain, destination: Chain) => Effect.Effect<Token>>(),
  }),
  accessors: true,
}) {
  static Test = Layer.effect(
    this,
    Effect.gen(function*() {
      const fc = yield* Effect.tryPromise({
        try: () => import("effect/FastCheck"),
        catch: (cause) =>
          new TokenRegistryError({
            message: `Could not import "effect/FastCheck"`,
            cause,
          }),
      })

      const Arbitrary = yield* Effect.tryPromise({
        try: () => import("effect/Arbitrary"),
        catch: (cause) =>
          new TokenRegistryError({
            message: `Could not import "effect/Arbitrary"`,
            cause,
          }),
      })

      const ArbitraryToken = Arbitrary.make(Token)

      return TokenRegistry.make(
        {
          pickQuote: (source: Chain, destination: Chain) =>
            pipe(
              Hash.string(source.universal_chain_id),
              Hash.combine(Hash.string(destination.universal_chain_id)),
              (seed) =>
                fc.sample(ArbitraryToken, {
                  numRuns: 1,
                  seed,
                })[0],
              Effect.succeed,
            ),
        },
      )
    }),
  )
}
