import { describe, it } from "@effect/vitest"
import { TokenRepresentation } from "@unionlabs/sdk/schema/token"
import { Arbitrary, Effect, FastCheck as fc, Struct } from "effect"
import { Pricing } from "../src/Pricing.js"

describe("Pricing", () => {
  it.layer(Pricing.Pyth)("Pyth", (it) =>
    it.effect("sanity", () =>
      Effect.gen(function*() {
        const tokenArb = Arbitrary.make(TokenRepresentation)
        const token = Struct.evolve(
          fc.sample(tokenArb, 1)[0],
          { symbol: () => "WETH" as const },
        )
        const pricing = yield* Pricing
        const result = yield* pricing.of(token)
        console.log(`WETH ≈ $${JSON.stringify(result, null, 2)} USD`)
      })))
})
