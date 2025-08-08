/**
 * This module sources {@link Channel} data.
 *
 * @since 2.0.0
 */

import { Data, Effect, Hash, hole, Layer, pipe } from "effect"
import { Chain } from "./schema/chain.js"
import { Channel } from "./schema/channel.js"

/**
 * @category errors
 * @since 2.0.0
 */
export class ChannelRegistryError
  extends Data.TaggedError("@unionlabs/sdk/ChannelRegistry/ChannelRegistryError")<{
    message: string
    cause?: unknown
  }>
{}

/**
 * @category Services
 * @since 2.0.0
 */
export class ChannelRegistry
  extends Effect.Service<ChannelRegistry>()("@unionlabs/sdk/ChannelRegistry", {
    sync: () => ({
      pick: hole<(source: Chain, destination: Chain) => Effect.Effect<Channel>>(),
    }),
    accessors: true,
  })
{
  static Test = Layer.effect(
    this,
    Effect.gen(function*() {
      const fc = yield* Effect.tryPromise({
        try: () => import("effect/FastCheck"),
        catch: (cause) =>
          new ChannelRegistryError({
            message: `Could not import "effect/FastCheck"`,
            cause,
          }),
      })

      const Arbitrary = yield* Effect.tryPromise({
        try: () => import("effect/Arbitrary"),
        catch: (cause) =>
          new ChannelRegistryError({
            message: `Could not import "effect/Arbitrary"`,
            cause,
          }),
      })

      const ArbitraryChannel = Arbitrary.make(Channel)

      return ChannelRegistry.make(
        {
          pick: (source: Chain, destination: Chain) =>
            pipe(
              Hash.string(source.universal_chain_id),
              Hash.combine(Hash.string(destination.universal_chain_id)),
              (seed) =>
                fc.sample(ArbitraryChannel, {
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
