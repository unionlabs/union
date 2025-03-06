import { Schema } from "effect"
import { Hex } from "$lib/schema/hex"
import { UniversalChainId } from "./chain.ts"
import { ChannelId } from "./channel.ts"

export const TokenRawDenom = Hex.pipe(Schema.brand("TokenRawDenom"))
export type TokenRawDenom = typeof TokenRawDenom.Type

export const TokenRawAmount = Schema.BigInt.pipe(Schema.brand("TokenRawAmount"))

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

export const RawTokenBalance = Schema.Option(TokenRawAmount).pipe(Schema.brand("RawTokenBalance"))
export type RawTokenBalance = typeof RawTokenBalance.Type
