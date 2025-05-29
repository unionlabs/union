import { describe, it } from "@effect/vitest"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { Effect, Layer, Logger } from "effect"
import { PriceOracle } from "../src/PriceOracle.js"
import { IN_NIX_BUILD } from "./utils.js"

const LoggerTest = Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)

describe("Pricing", () => {
  it.layer(Layer.mergeAll(PriceOracle.Test, LoggerTest))(
    "Test",
    (it) =>
      it.effect("scratchpad", () =>
        Effect.gen(function*() {
          const id = UniversalChainId.make("ethereum.11155111")
          const pricing = yield* PriceOracle
          const result = yield* pricing.of(id)
          console.log(`[TEST] WETH to USD: ${JSON.stringify(result, null, 2)}`)
        })),
  )

  it.layer(Layer.mergeAll(PriceOracle.Pyth, LoggerTest))(
    "Pyth",
    (it) =>
      it.effect.skipIf(IN_NIX_BUILD)("scratchpad", () =>
        Effect.gen(function*() {
          const id = UniversalChainId.make("ethereum.11155111")
          const pricing = yield* PriceOracle
          const result = yield* pricing.of(id)
          console.log(`[PYTH] WETH to USD: ${JSON.stringify(result, null, 2)}`)
        })),
  )
})
