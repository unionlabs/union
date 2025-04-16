import { Schema } from "effect"
import { Hex } from "./hex.js"
import { UniversalChainId } from "./chain.js"
import { ChannelId } from "./channel.js"
import { AddressEvmCanonical } from "./address.js"

export const TokenRawDenom = Hex.pipe(Schema.brand("TokenRawDenom"))
export type TokenRawDenom = typeof TokenRawDenom.Type

export const CosmosBankDenom = Schema.String.pipe(Schema.brand("CosmosBankDenom"))
export type CosmosBankDenom = typeof CosmosBankDenom.Type

export const TokenRawAmount = Schema.BigInt.pipe(Schema.brand("TokenRawAmount"))
export type TokenRawAmount = typeof TokenRawAmount.Type

export const TokenRawAmountFromSelf = Schema.BigIntFromSelf.pipe(Schema.brand("TokenRawAmount"))
export type TokenRawAmountFromSelf = typeof TokenRawAmountFromSelf.Type

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
  source: TokenSource
}) {}

export class TokenRepresentation extends Schema.Class<TokenRepresentation>("TokenRepresentation")({
  logo_uri: Schema.OptionFromNonEmptyTrimmedString,
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
  rank: Schema.OptionFromNullOr(Schema.Int),
  denom: TokenRawDenom,
  representations: Schema.Array(TokenRepresentation),
  wrapping: Schema.Array(TokenWrapping)
}) {}

export const Tokens = Schema.Array(Token)
export type Tokens = typeof Tokens.Type

export const RawTokenBalance = Schema.Option(TokenRawAmount).pipe(Schema.brand("RawTokenBalance"))
export type RawTokenBalance = typeof RawTokenBalance.Type

export const EvmWethToken = AddressEvmCanonical.pipe(
  Schema.annotations({
    message: () =>
      "WETH token must be a valid Evm canonical address (e.g., 0x followed by 40 hex chars)"
  })
)
export type EvmWethToken = typeof EvmWethToken.Type

export const QuoteData = Schema.Union(
  Schema.Struct({
    quote_token: Hex,
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
  }),
  Schema.Struct({ type: Schema.Literal("QUOTE_ERROR"), cause: Schema.String })
)
export type QuoteData = typeof QuoteData.Type

export const WethTokenData = Schema.Union(
  Schema.Struct({ wethQuoteToken: Hex }),
  Schema.Struct({ type: Schema.Literal("NOT_EVM") }),
  Schema.Struct({ type: Schema.Literal("NO_WETH_QUOTE") }),
  Schema.Struct({ type: Schema.Literal("WETH_LOADING") }),
  Schema.Struct({ type: Schema.Literal("WETH_MISSING_ARGUMENTS") }),
  Schema.Struct({ type: Schema.Literal("WETH_ERROR"), cause: Schema.String })
)
export type WethTokenData = typeof WethTokenData.Type

//Change to
// export type QuoteData = Data.TaggedEnum<{
//   UNWRAPPED: { quote_token: Schema.String }
//   NEW_WRAPPED: { quote_token: Schema.String }
//   NO_QUOTE_AVAILABLE: {}
//   QUOTE_LOADING: {}
//   QUOTE_MISSING_ARGUMENTS: {}
//   QUOTE_ERROR: { cause: Schema.String }
// }>
//
// export type WethTokenData = Data.TaggedEnum<{
//   WETH_QUOTE: { wethQuoteToken: Schema.String }
//   NO_WETH_QUOTE: {}
//   WETH_LOADING: {}
//   WETH_MISSING_ARGUMENTS: {}
//   WETH_ERROR: { cause: Schema.String }
// }>
