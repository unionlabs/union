import { describe, it, assert } from "@effect/vitest"
import { Chain } from "@unionlabs/sdk/schema/chain"
import { Arbitrary, FastCheck as fc } from "effect"

const chainArb = Arbitrary.make(Chain)
const cosmosChainArb = chainArb.filter(x => x.rpc_type === "cosmos")
const evmChainArb = chainArb.filter(x => x.rpc_type === "evm")

describe("Transfer Args", () => {
  it("produces a chain", () => {
    const chain = fc.sample(chainArb, 1)[0]
    console.log({ chain })
    assert.isTrue(chain instanceof Chain)
  })

  it("discriminates", () => {

  })
})
