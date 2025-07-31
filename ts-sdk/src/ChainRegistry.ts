import { Data, Effect, Hash, hole, Layer, pipe } from "effect"
import { Chain, UniversalChainId } from "./schema/chain.js"

export class ChainRegistryError
  extends Data.TaggedError("@unionlabs/sdk/ChainRegistry/ChainRegistryError")<{
    message: string
    cause?: unknown
  }>
{}

export class ChainRegistry extends Effect.Service<ChainRegistry>()("@unionlabs/sdk/ChainRegistry", {
  sync: () => ({
    byUniversalId: hole<(id: UniversalChainId) => Effect.Effect<Chain>>(),
  }),
  accessors: true,
}) {
  static Test = Layer.effect(
    this,
    Effect.gen(function*() {
      const fc = yield* Effect.tryPromise({
        try: () => import("effect/FastCheck"),
        catch: (cause) =>
          new ChainRegistryError({
            message: `Could not import "effect/FastCheck"`,
            cause,
          }),
      })

      const Arbitrary = yield* Effect.tryPromise({
        try: () => import("effect/Arbitrary"),
        catch: (cause) =>
          new ChainRegistryError({
            message: `Could not import "effect/Arbitrary"`,
            cause,
          }),
      })

      const ArbitraryChain = Arbitrary.make(Chain)

      return ChainRegistry.make(
        {
          byUniversalId: (id: UniversalChainId) =>
            pipe(
              Hash.string(id),
              (seed) =>
                fc.sample(ArbitraryChain, {
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
