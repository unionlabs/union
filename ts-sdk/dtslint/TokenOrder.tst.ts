import { Chain } from "@unionlabs/sdk/schema/chain"
import * as Token from "@unionlabs/sdk/Token"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import { Effect, pipe } from "effect"
import { ParseError } from "effect/ParseResult"
import { describe, expect, it } from "tstyche"

declare const chain: Chain

describe("TokenOrder", () => {
  it("complete", () => {
    expect(TokenOrder.make({
      source: chain,
      destination: chain,
      sender: Ucs05.EvmDisplay.make("0x06627714f3F17a701f7074a12C02847a5D2Ca487"),
      receiver: Ucs05.CosmosDisplay.make("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"),
      // USDC on Sepolia
      baseToken: Token.Erc20.make({ address: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238" }),
      kind: TokenOrder.Kind.Escrow,
      baseAmount: 100n,
      quoteAmount: 100n,
      quoteToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
      metadata: "0x",
    }))
      .type
      .toBe<
        Effect.Effect<TokenOrder.TokenOrder.Complete, ParseError, never>
      >()
  })
  it("partial", () => {
    expect(TokenOrder.make({
      source: chain,
      destination: chain,
      sender: Ucs05.EvmDisplay.make("0x06627714f3F17a701f7074a12C02847a5D2Ca487"),
      receiver: Ucs05.CosmosDisplay.make("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"),
      // USDC on Sepolia
      baseToken: Token.Erc20.make({ address: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238" }),
      kind: TokenOrder.Kind.Escrow,
      baseAmount: 100n,
    }))
      .type
      .toBe<
        Effect.Effect<
          TokenOrder.TokenOrder.Build<"quoteToken" | "quoteAmount" | "metadata">,
          ParseError,
          never
        >
      >()
  })
  it("partial w/ singular fulfillment", () => {
    expect(pipe(
      TokenOrder.make({
        source: chain,
        destination: chain,
        sender: Ucs05.EvmDisplay.make("0x06627714f3F17a701f7074a12C02847a5D2Ca487"),
        receiver: Ucs05.CosmosDisplay.make("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"),
        // USDC on Sepolia
        baseToken: Token.Erc20.make({ address: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238" }),
        kind: TokenOrder.Kind.Escrow,
        quoteAmount: 100n,
        baseAmount: 100n,
        metadata: "0x",
      }),
      Effect.tap((x) =>
        expect(x).type.toBe<
          TokenOrder.TokenOrder.Build<"quoteToken">
        >()
      ),
      Effect.flatMap(TokenOrder.withAutoQuoteToken),
    ))
      .type
      .toBe<
        Effect.Effect<
          TokenOrder.TokenOrder.Complete,
          ParseError,
          "quote registry"
        >
      >()
  })
})
