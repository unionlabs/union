import { assert, describe, it } from "@effect/vitest";
import { Effect, Schema } from "effect";
import * as Address from "@unionlabs/sdk/schema/address"

describe("Address", () => {
  describe("Transforms", () => {
    it.effect("cosmos", () => Effect.gen(function*() {
      const display = Address.AddressCosmosDisplay.make("cosmos1xzp40d2j3egul44nu0rurdug8ztpxdcch794tx")
      const canonical = "0x308357b5528e51cfd6b3e3c7c1b7883896133718"
      const displayFromCanonical = yield* Schema.decode(Address.AddressCosmosDisplayFromHex)(canonical)
      const canonicalFromDisplay = yield* Schema.encode(Address.AddressCosmosDisplayFromHex)(display)

      assert.strictEqual(display, displayFromCanonical)
      assert.strictEqual(canonical, canonicalFromDisplay)
    }))
  })
})
