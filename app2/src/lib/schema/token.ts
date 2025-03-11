import { Schema } from "effect"
import { Hex } from "$lib/schema/hex"
import { UniversalChainId } from "./chain.ts"
import { ChannelId } from "./channel.ts"
import { AddressEvmCanonical } from "$lib/schema/address.ts"

export const TokenRawDenom = Hex.pipe(Schema.brand("TokenRawDenom"))
export type TokenRawDenom = typeof TokenRawDenom.Type

export const CosmosBankDenom = Schema.String.pipe(Schema.brand("CosmosBankDenom"))
export type CosmosBankDenom = typeof CosmosBankDenom.Type

export const TokenRawAmount = Schema.BigInt.pipe(Schema.brand("TokenRawAmount"))
export type TokenRawAmount = typeof TokenRawAmount.Type

export class TokenCw20 extends Schema.Class<TokenCw20>("TokenCw20")({
  cw20_token_address: Schema.String
}) {}

export class TokenSource extends Schema.Class<TokenSource>("TokenSource")({
  name: Schema.String,
  logo_uri: Schema.OptionFromNullOr(Schema.String),
  source_uri: Schema.String
}) {}

export class TokenSourceWrapping extends Schema.Class<TokenSourceWrapping>("TokenSourceWrapping")({
  destination_channel_id: ChannelId,
  unwrapped_chain: Schema.Struct({
    universal_chain_id: UniversalChainId
  }),
  unwrapped_denom: TokenRawDenom
}) {}

export class TokenSourceInfo extends Schema.Class<TokenSourceInfo>("TokenSourceInfo")({
  update_timestamp: Schema.DateTimeUtc,
  source: TokenSource,
  wrapping: Schema.Array(TokenSourceWrapping)
}) {}

export class TokenRepresentation extends Schema.Class<TokenRepresentation>("TokenRepresentation")({
  name: Schema.String,
  symbol: Schema.String,
  decimals: Schema.Number,
  sources: Schema.Array(TokenSourceInfo)
}) {}

export class TokenWrapping extends Schema.Class<TokenWrapping>("TokenWrapping")({
  unwrapped_chain: Schema.Struct({
    universal_chain_id: UniversalChainId
  }),
  wrapped_chain: Schema.Struct({
    universal_chain_id: UniversalChainId
  }),
  destination_channel_id: ChannelId,
  unwrapped_denom: TokenRawDenom
}) {}

export class Token extends Schema.Class<Token>("Token")({
  denom: TokenRawDenom,
  cw20: Schema.OptionFromNullOr(TokenCw20),
  representations: Schema.Array(TokenRepresentation),
  wrapping: Schema.Array(TokenWrapping)
}) {}

export const Tokens = Schema.Array(Token)
export type Tokens = typeof Tokens.Type

export const RawTokenBalance = Schema.Option(TokenRawAmount).pipe(Schema.brand("RawTokenBalance"))
export type RawTokenBalance = typeof RawTokenBalance.Type

export const EVMWethToken = AddressEvmCanonical.pipe(
  Schema.annotations({
    message: () =>
      "WETH token must be a valid EVM canonical address (e.g., 0x followed by 40 hex chars)"
  })
)

export const QuoteData = Schema.Union(
  Schema.Struct({
    quote_token: Schema.String,
    type: Schema.Literal("UNWRAPPED", "NEW_WRAPPED")
  }),
  Schema.Struct({
    type: Schema.Literal("NO_QUOTE_AVAILABLE")
  }),
  Schema.Struct({
    type: Schema.Literal("QUOTE_LOADING")
  }),
  Schema.Struct({
    type: Schema.Literal("QUOTE_MISSING_ARGUMENTS")
  })
)

export const WethTokenData = Schema.Union(
  Schema.Struct({ wethQuoteToken: Schema.String }),
  Schema.Struct({ type: Schema.Literal("NO_WETH_QUOTE") }),
  Schema.Struct({ type: Schema.Literal("WETH_LOADING") }),
  Schema.Struct({ type: Schema.Literal("WETH_MISSING_ARGUMENTS") })
)
