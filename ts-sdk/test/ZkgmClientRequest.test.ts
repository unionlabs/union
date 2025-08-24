import { assert, describe, it } from "@effect/vitest"
import * as Batch from "@unionlabs/sdk/Batch"
import { Chain } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import * as Token from "@unionlabs/sdk/Token"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import * as ZkgmClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import { Arbitrary, Effect, pipe } from "effect"
import * as A from "effect/Array"
import * as fc from "effect/FastCheck"
import * as O from "effect/Option"
import * as Tuple from "effect/Tuple"

describe("ZkgmClientRequest", () => {
  it.effect("requiredFunds", () =>
    Effect.gen(function*() {
      const [source, destination] = fc.sample(Arbitrary.make(Chain), 2)

      const order = yield* TokenOrder.make({
        source,
        destination,
        sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
        receiver: "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
        baseToken: "muno",
        kind: "escrow",
        baseAmount: 100n,
        quoteToken: "muno",
        quoteAmount: 100n,
        metadata: undefined,
        version: undefined,
      })

      const request = ZkgmClientRequest.make({
        source,
        destination,
        channelId: ChannelId.make(2),
        ucs03Address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
        instruction: order,
      })

      const requiredFunds = ZkgmClientRequest.requiredFunds(request)

      assert.deepStrictEqual(
        requiredFunds,
        O.some(A.make(
          Tuple.make(Token.CosmosBank.make({ address: "muno" }), 100n),
        )),
      )
    }))
  it.effect.skip("batch requiredFunds", () =>
    Effect.gen(function*() {
      const [source, destination] = fc.sample(Arbitrary.make(Chain), 2)

      const batch = Batch.make([
        yield* TokenOrder.make({
          source,
          destination,
          sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
          receiver: "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
          baseToken: "muno",
          kind: "escrow",
          baseAmount: 100n,
          quoteToken: "muno",
          quoteAmount: 100n,
          metadata: undefined,
          version: undefined,
        }),
        yield* TokenOrder.make({
          source,
          destination,
          sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
          receiver: "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
          baseToken: "0x685ce6742351ae9b618f383883d6d1e0c5a31b4b",
          kind: "escrow",
          baseAmount: 100n,
          quoteToken: "muno",
          quoteAmount: 100n,
          metadata: undefined,
          version: undefined,
        }),
        yield* TokenOrder.make({
          source,
          destination,
          sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
          receiver: "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
          baseToken: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
          kind: "escrow",
          baseAmount: 100n,
          quoteToken: "muno",
          quoteAmount: 100n,
          metadata: undefined,
          version: undefined,
        }),
      ])

      const request = ZkgmClientRequest.make({
        source,
        destination,
        channelId: ChannelId.make(2),
        ucs03Address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
        instruction: batch,
      })

      const requiredFunds = ZkgmClientRequest.requiredFunds(request)

      assert.deepStrictEqual(
        requiredFunds,
        O.some(A.make(
          Tuple.make(
            Token.EvmGas.make({ address: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE" }),
            100n,
          ),
          Tuple.make(Token.CosmosBank.make({ address: "muno" }), 100n),
          Tuple.make(
            Token.Erc20.make({ address: "0x685ce6742351ae9b618f383883d6d1e0c5a31b4b" }),
            100n,
          ),
        )),
      )

      const nativeFunds = pipe(
        requiredFunds,
        O.map(A.filter(([token, amount]) => Token.isNative(token))),
      )

      assert.deepStrictEqual(
        nativeFunds,
        O.some(A.make(
          Tuple.make(Token.CosmosBank.make({ address: "muno" }), 100n),
          Tuple.make(
            Token.EvmGas.make({ address: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE" }),
            100n,
          ),
        )),
      )
    }))
})
