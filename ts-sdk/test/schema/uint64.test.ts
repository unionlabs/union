import { describe, it } from "vitest"
import * as fc from "effect/FastCheck"
import * as Arbitrary from "effect/Arbitrary"
import { Uint64 } from "../../src/schema/uint64.js"

describe("Uint64", () => {
  it("within range", () =>
    fc.assert(
      fc.property(Arbitrary.make(Uint64), x => x >= 0n && x <= 18_446_744_073_709_551_615n)
    ))
})
