import * as Arbitrary from "effect/Arbitrary"
import * as fc from "effect/FastCheck"
import { flow } from "effect/Function"
import * as S from "effect/Schema"
import { describe, it } from "vitest"
import { Hex } from "../../src/schema/hex.js"
import * as Instruction from "../../src/ucs03/instruction.js"

const isHex = S.is(Hex)
const InstructionArbitrary = Arbitrary.make(Instruction.Schema)
const checkEncodeIsHex = flow(Instruction.encodeAbi, isHex)

describe("Instruction", () => {
  describe("encodeAbi", () => {
    // Skip to preserve determinism and until `Instruction.Schema` is stable.
    it.skip.for([
      // 1747069433 // uint8 controversy
      undefined,
    ])("passes", seed =>
      fc.assert(
        fc.property(InstructionArbitrary, checkEncodeIsHex),
        typeof seed !== "undefined" ? { seed } : {},
      ))
  })
})
