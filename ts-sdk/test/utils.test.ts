import { assert, describe, it } from "@effect/vitest"
import { generateSalt, verifySalt } from "@unionlabs/sdk/utils"
import { Effect } from "effect"

describe("Utils", () => {
  it.effect.each(["evm", "cosmos"] as const)("computes CRC-verifiable salt", x =>
    Effect.gen(function* () {
      const salt = yield* generateSalt(x)
      const verification = yield* verifySalt(salt)
      assert.isTrue(verification)
    })
  )
})
