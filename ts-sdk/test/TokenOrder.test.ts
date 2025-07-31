import { describe, it } from "@effect/vitest"
import { Chain } from "@unionlabs/sdk/schema/chain"
import * as Token from "@unionlabs/sdk/Token"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import { Arbitrary, Effect } from "effect"
import * as fc from "effect/FastCheck"

const ArbitraryChain = Arbitrary.make(Chain)

describe("TokenOrder", () => {
  it.effect("constructs", () =>
    Effect.gen(function*() {
      const [source, destination] = fc.sample(ArbitraryChain, 2)
      const incompleteTokenOrder = yield* TokenOrder.make({
        source,
        destination,
        sender: Ucs05.EvmDisplay.make("0x06627714f3F17a701f7074a12C02847a5D2Ca487"),
        receiver: Ucs05.CosmosDisplay.make("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"),
        // USDC on Sepolia
        baseToken: Token.Erc20.make({ address: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238" }),
        kind: TokenOrder.Kind.Escrow,
        baseAmount: 100n,
        // USDC on Sepolia
        quoteToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
      })
    }))
})
