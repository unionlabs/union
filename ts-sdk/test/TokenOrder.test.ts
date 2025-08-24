import { describe, it } from "@effect/vitest"
import { Chain } from "@unionlabs/sdk/schema/chain"
import * as Token from "@unionlabs/sdk/Token"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import { TokenRegistry } from "@unionlabs/sdk/TokenRegistry"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import { Arbitrary, Effect } from "effect"
import * as fc from "effect/FastCheck"

const ArbitraryChain = Arbitrary.make(Chain)

describe("TokenOrder", () => {
  it.layer(TokenRegistry.Test)("constructs", (it) => {
    it.effect.skip("auto quote token", () =>
      Effect.gen(function*() {
        const [source, destination] = fc.sample(ArbitraryChain, 2)
        const order = yield* TokenOrder.make({
          source,
          destination,
          sender: Ucs05.EvmDisplay.make({ address: "0x06627714f3F17a701f7074a12C02847a5D2Ca487" }),
          receiver: Ucs05.CosmosDisplay.make({
            address: "bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh",
          }),
          // USDC on Sepolia
          baseToken: Token.Erc20.make({ address: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238" }),
          kind: "escrow",
          baseAmount: 100n,
          // USDC on Sepolia
          quoteAmount: 100n,
          metadata: undefined,

          version: undefined,
        })

        const completed = yield* order.pipe(
          TokenOrder.withAutoQuoteToken,
        )

        console.log({ completed })
      }))
  })

  // it.effect.skip("test_create_foa_v2_image_evm", () =>
  //   Effect.gen(function*() {
  //     const [source, destination] = fc.sample(ArbitraryChain, 2)
  //     const order = yield* TokenOrder.make({
  //       source,
  //       destination,
  //       sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
  //       receiver: "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
  //       baseToken: "muno",
  //       kind: TokenOrder.Kind.Escrow,
  //       baseAmount: 100n,
  //       quoteToken: "0x49aCf968c7E8807B39e980b2a924E97C8ead3a22",
  //       quoteAmount: 100n,
  //       metadata: "0x996be231a091877022ccdbf41da6e2f92e097c0ccc9480f8b3c630e5c2b14ff1",
  //       version: undefined,
  //     })

  //     const encoded = yield* order.encode
  //     const decoded = yield* Schema.decode(Ucs03.Ucs03FromHex)(encoded)

  //     console.log({ order, encoded, decoded })

  //     assert.equal(
  //       encoded,
  //       "0x00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001449acf968c7e8807b39e980b2a924e97c8ead3a220000000000000000000000000000000000000000000000000000000000000000000000000000000000000020996be231a091877022ccdbf41da6e2f92e097c0ccc9480f8b3c630e5c2b14ff1",
  //     )
  //   }))
})
