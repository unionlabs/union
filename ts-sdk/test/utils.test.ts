import { assert, beforeEach, describe, it, vi } from "@effect/vitest"
import { generateSalt, verifySalt } from "@unionlabs/sdk/utils"
import { Effect, Either } from "effect"

describe.sequential("Utils", () => {
  beforeEach(() => {
    vi.unstubAllGlobals()
  })
  describe("with crypto", () => {
    it.effect.each(["evm", "cosmos"] as const)("computes verifiable salt", x =>
      Effect.gen(function* () {
        const salt = yield* generateSalt(x)
        const verification = yield* verifySalt(salt)
        assert.isTrue(verification)
      })
    )
  })

  describe("without crypto", () => {
    vi.stubGlobal("crypto", undefined)
    it.effect.each(["evm", "cosmos"] as const)("fails to compute salt", x =>
      Effect.gen(function* () {
        const result = yield* Effect.either(generateSalt(x))
        assert.isTrue(Either.isRight(result))
      })
    )
  })
})
