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
            JSON.stringify(obj),
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
            () => JSON.stringify(obj),
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
      const s = `{"a":0,"b":"b","c":"<circular>"}`
      expect(f(obj)).toStrictEqual(s)
    })
  })
})
