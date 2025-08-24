/**
 * This module estimates cross-chain transaction fees.
 *
 * @since 2.0.0
 */
import { Data, Effect, Hash, hole, Layer, pipe } from "effect"
import * as PriceOracle from "./PriceOracle.js"
import { Chain } from "./schema/chain.js"

/**
 * @category errors
 * @since 2.0.0
 */
export class FeeEstimatorError
  extends Data.TaggedError("@unionlabs/sdk/FeeEstimator/FeeEstimatorError")<{
    message: string
    cause?: unknown
  }>
{}

/**
 * @category services
 * @since 2.0.0
 */
export class FeeEstimator extends Effect.Service<FeeEstimator>()("@unionlabs/sdk/FeeEstimator", {
  effect: Effect.gen(function*() {
    const oracle = yield* PriceOracle.PriceOracle
    yield* Effect.log(oracle)
    return {
      calculate: hole<(source: Chain, destination: Chain) => Effect.Effect<any>>(),
    } as const
  }),
  dependencies: [PriceOracle.PriceOracleExecutor.Default],
  accessors: true,
}) {
  static Test = Layer.effect(
    this,
    Effect.gen(function*() {
      const fc = yield* Effect.tryPromise({
        try: () => import("effect/FastCheck"),
        catch: (cause) =>
          new FeeEstimatorError({
            message: `Could not import "effect/FastCheck"`,
            cause,
          }),
      })

      const Arbitrary = yield* Effect.tryPromise({
        try: () => import("effect/Arbitrary"),
        catch: (cause) =>
          new FeeEstimatorError({
            message: `Could not import "effect/Arbitrary"`,
            cause,
          }),
      })

      const ArbitraryFeeEstimator = Arbitrary.make(Chain)

      return FeeEstimator.make(
        {
          calculate: (source: Chain, destination: Chain) =>
            pipe(
              Hash.string(source.universal_chain_id),
              Hash.combine(Hash.string(destination.universal_chain_id)),
              (seed) =>
                fc.sample(ArbitraryFeeEstimator, {
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
