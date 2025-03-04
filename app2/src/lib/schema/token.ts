import { Schema } from "effect"
import { Hex } from "$lib/schema/hex"
import { ChainId } from "./chain"
import { ChannelId } from "./channel"

export const TokenRawDenom = Hex.pipe(Schema.brand("TokenRawDenom"))
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
    chain_id: ChainId
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
    chain_id: ChainId
  }),
  wrapped_chain: Schema.Struct({
    chain_id: ChainId
  }),
  destination_channel_id: ChannelId,
  unwrapped_denom: Schema.String
}) {}

export class Token extends Schema.Class<Token>("Token")({
  denom: TokenRawDenom,
  cw20: Schema.OptionFromNullOr(TokenCw20),
  chain: Schema.Struct({
    chain_id: ChainId
  }),
  representations: Schema.Array(TokenRepresentation),
  wrapping: Schema.Array(TokenWrapping)
}) {}

export const Tokens = Schema.Array(Token)
