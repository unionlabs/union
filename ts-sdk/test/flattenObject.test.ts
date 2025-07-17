import { assert, describe, it } from "@effect/vitest"
import { Schema as S, Struct } from "effect"
import * as Staking from "../src/Staking.js"
import { flattenObject } from "../src/utils/flattenObject.js"

describe("flattenObject", () => {
  it("thing", () => {
    const obj = S.decodeSync(Staking.GetValidators)({
      _tag: "GetValidators",
      status: "BOND_STATUS_BONDED",
      pagination: {
        key: "mykey",
        offset: undefined,
      },
    })

    const result = flattenObject(Struct.omit(obj, "_tag"))

    assert.deepStrictEqual(
      result,
      {
        "status": "BOND_STATUS_BONDED",
        "pagination.key": "mykey",
      },
    )
  })
})
