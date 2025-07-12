import { describe, it } from "@effect/vitest"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { BigDecimal, Effect, Layer, Logger } from "effect"
import { Band, PriceOracle, PriceOracleExecutor, Pyth, Redstone } from "../src/PriceOracle.js"
import { IN_NIX_BUILD } from "./utils.js"

const LoggerTest = Logger.replace(
  Logger.defaultLogger,
  Logger.prettyLogger({
    colors: true,
    mode: "tty",
    stderr: true,
  }),
)

describe("PriceOracle (Test)", () => {
  it.layer(Layer.mergeAll(PriceOracleExecutor.Test, LoggerTest))(
    "Test",
    (it) =>
      it.effect.skip("of", () =>
        Effect.gen(function*() {
          const id = UniversalChainId.make("osmosis.osmo-test-5")
          const pricing = yield* PriceOracle
          const result = yield* pricing.of(id)
          console.log(`[TEST] OSMO to USD: ${JSON.stringify(result, null, 2)}`)
        })),
  )
})

describe.skipIf(IN_NIX_BUILD)("PriceOracle (Live)", () => {
  it.effect.each(
    [
      ["Pyth", Pyth],
      ["Redstone", Redstone],
      ["Band", Band],
    ] as const,
  )("%s > of", ([label, layer]) =>
    Effect.gen(function*() {
      const id = UniversalChainId.make("ethereum.11155111")
      const pricing = yield* PriceOracle.pipe(
        Effect.provide(layer),
      )
      yield* pricing.of(id).pipe(
        Effect.map(x => BigDecimal.format(x.price)),
        Effect.tap((a) => Effect.log(label, id, a)),
        Effect.tapErrorCause((cause) => Effect.log("fail", cause)),
      )
    }).pipe(Effect.provide(LoggerTest)))

  it.layer(Layer.mergeAll(PriceOracleExecutor.Default, LoggerTest))(
    "Executor",
    (it) =>
      it.effect("of", () =>
        Effect.gen(function*() {
          const id = UniversalChainId.make("ethereum.11155111")
          const pricing = yield* PriceOracle
          const result = yield* pricing.of(id).pipe(
            Effect.tap((a) => Effect.log("success", a)),
            Effect.tapErrorCause((cause) => Effect.log("fail", cause)),
          )
          console.log(`ETH to USD: ${JSON.stringify(result, null, 2)}`)
        })),
  )
})
