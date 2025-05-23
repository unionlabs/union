import { Effect, flow } from "effect"
import * as fc from "effect/FastCheck"
import { assert, describe, expect, it } from "vitest"
import { safeStringifyJSON } from "./json"

describe("JSON Utilities", () => {
  describe("safeStringifyJSON", () => {
    it("stringifies normal JSON", () => {
      fc.assert(fc.property(
        fc.object(),
        (obj) => {
          assert.strictEqual(
            JSON.stringify(obj, null, 2),
            safeStringifyJSON(obj).pipe(Effect.runSync),
          )
        },
      ))
    })

    it("stringifies circular JSON", () => {
      fc.assert(fc.property(
        fc.object(),
        (obj) => {
          obj.circular = obj
          assert.throw(
            () => JSON.stringify(obj, null, 2),
          )
          assert.doesNotThrow(
            () => safeStringifyJSON(obj).pipe(Effect.runSync),
          )
        },
      ))
    })

    it("halts on circular JSON with terminal identifier", () => {
      const obj = {
        a: 0,
        b: "b",
        c: {},
      } as object
      // @ts-ignore
      obj["c"] = obj
      const f = flow(safeStringifyJSON, Effect.runSync)
      const s = `{\n  "a": 0,\n  "b": "b",\n  "c": "<circular>"\n}`
      expect(f(obj)).toStrictEqual(s)
    })

    it("halts on circular JSON with terminal identifier", () => {
      const obj = { a: {}, b: {}, c: {} }
      // @ts-ignore
      obj.a.self = obj.a // a → a
      // @ts-ignore
      obj.b.ref = obj // b → root → b (cycle through root)
      // @ts-ignore
      obj.c.deep = { loop: obj.c } // c → deep → c
      const f = flow(safeStringifyJSON, Effect.runSync)
      const s =
        `{\n  "a": {\n    "self": "<circular>"\n  },\n  "b": {\n    "ref": "<circular>"\n  },\n  "c": {\n    "deep": {\n      "loop": "<circular>"\n    }\n  }\n}`
      expect(f(obj)).toStrictEqual(s)
    })
  })
})
