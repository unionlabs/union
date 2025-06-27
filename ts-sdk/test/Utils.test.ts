import { assert, beforeEach, describe, it, vi } from "@effect/vitest"
import { Utils } from "@unionlabs/sdk"
import { Effect, Either } from "effect"

describe.sequential("Utils", () => {
  beforeEach(() => {
    vi.unstubAllGlobals()
  })
  describe("with crypto", () => {
    it.effect.each(["evm", "cosmos"] as const)(
      "computes verifiable salt",
      x =>
        Effect.gen(function*() {
          const salt = yield* Utils.generateSalt(x)
          const verification = yield* Utils.verifySalt(salt)
          assert.isTrue(verification)
        }),
    )
  })

  describe("without crypto", () => {
    vi.stubGlobal("crypto", undefined)
    it.effect.each(["evm", "cosmos"] as const)(
      "fails to compute salt",
      x =>
        Effect.gen(function*() {
          const result = yield* Effect.either(Utils.generateSalt(x))
          assert.isTrue(Either.isRight(result))
        }),
    )
  })
})
