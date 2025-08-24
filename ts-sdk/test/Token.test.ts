import { assert, describe, it } from "@effect/vitest"
import * as Token from "@unionlabs/sdk/Token"
import { Effect, Schema as S } from "effect"

describe("Token", () => {
  describe("Erc20", () => {
    it.each(
      [
        "0xA0b86991C6218b36c1d19D4a2e9Eb0cE3606eB48",
      ] as const,
    )("constructs %s", (address) => {
      assert.deepStrictEqual(
        Token.Erc20.make({ address }),
        {
          _tag: "Erc20",
          address,
        } as unknown as any,
      )
    })
  })
  describe("TokenFromString", () => {
    it.effect("transforms ERC20", () =>
      Effect.gen(function*() {
        const token = yield* S.decode(Token.TokenFromString)(
          "0xA0b86991C6218b36c1d19D4a2e9Eb0cE3606eB48",
        )
        assert.equal(token._tag, "Erc20")
      }))

    it.effect("transforms Cosmos Bank", () =>
      Effect.gen(function*() {
        const token = yield* S.decode(Token.TokenFromString)(
          "uatom",
        )
        assert.equal(token._tag, "CosmosBank")
      }))

    it.effect("transforms IBC classic", () =>
      Effect.gen(function*() {
        const token = yield* S.decode(Token.TokenFromString)(
          "ibc/B3504F0B84FEF5D8817A5196E19A886F81606DCD9D9FCA2E01B7F38379F94303",
        )
        assert.equal(token._tag, "CosmosIbcClassic")
      }))
  })
})
