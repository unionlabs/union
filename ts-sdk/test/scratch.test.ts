import { describe, it } from "@effect/vitest"
import { Effect } from "effect"
import * as S from "effect/Schema"

describe("scratch", () => {
  it.effect("hex", () =>
    Effect.gen(function*() {
      const f = S.decode(S.Uint8ArrayFromHex)
      const r = yield* f("0x123")
      yield* Effect.log(f)
    }))
})
