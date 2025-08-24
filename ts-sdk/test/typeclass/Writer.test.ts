import * as ArrayInstances from "@effect/typeclass/data/Array"
import * as StringInstances from "@effect/typeclass/data/String"
import * as FlatMap from "@effect/typeclass/FlatMap"
import { assert, describe, it } from "@effect/vitest"
import * as Writer from "@unionlabs/sdk/typeclass/Writer"
import * as A from "effect/Array"
import { pipe } from "effect/Function"

describe.concurrent("Writer", () => {
  describe("FlatMap", () => {
    it.each([
      [
        "getMonad",
        pipe(
          StringInstances.Monoid,
          Writer.getMonad,
          FlatMap.composeK,
        ),
      ],
      [
        "getFlatMap",
        pipe(
          Writer.getFlatMap(StringInstances.Monoid),
          FlatMap.composeK,
        ),
      ],
    ])("composeK (%s)", (_, composeK) => {
      const f = (s: string): [number, string] => [s.length, "[length]"]
      const g = (n: number): [boolean, string] => [n > 3, `[(>) 3 ${n}]`]
      const h = pipe(f, composeK(g))
      assert.deepStrictEqual(h(""), [false, "[length][(>) 3 0]"])
      assert.deepStrictEqual(h("abc"), [false, "[length][(>) 3 3]"])
      assert.deepStrictEqual(h("abcd"), [true, "[length][(>) 3 4]"])
    })

    it("mapWriter", () => {
      const composeK = pipe(
        Writer.getFlatMap(ArrayInstances.getMonoid<string>()),
        FlatMap.composeK,
      )
      const f = (s: string): [number, string[]] => [s.length, ["length"]]
      const g = (n: number): [boolean, string[]] => [n > 3, [">", "3", `${n}`]]
      const h = pipe(f, composeK(g))
      const j = Writer.mapWriter(
        (a: boolean, w: readonly string[]) => [a, pipe(w, A.reverse, A.join(" · "))],
      )
      assert.deepStrictEqual(j(h("a")), [false, "1 · 3 · > · length"])
    })
  })
})
