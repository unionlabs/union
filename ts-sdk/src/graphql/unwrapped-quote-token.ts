import { TokenRawDenom } from "../schema/token.js"
import type { UniversalChainId } from "../schema/chain.js"
import { fetchDecodeGraphql } from "../utils/graphql-query.js"
import { Effect, Schema, Struct, Array as Arr, flow, Option } from "effect"
import { graphql } from "gql.tada"
import type { ChannelId } from "../schema/channel.js"
import type { Hex } from "../schema/hex.js"

export const graphqlQuoteTokenUnwrapQuery = (args: {
  baseToken: Hex // token raw denom
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
}) =>
  fetchDecodeGraphql(
    Schema.Struct({
      v2_tokens: Schema.Array(
        Schema.Struct({
          wrapping: Schema.Array(Schema.Struct({ unwrapped_denom: TokenRawDenom }))
        })
      )
    }),
    graphql(/* GraphQL */ `
      query QueryTokenWrapping($source_chain_id: String!, $base_token: String!, $destination_channel_id: Int!) {
        v2_tokens(args: {p_universal_chain_id: $source_chain_id, p_denom: $base_token}) {
          denom
          wrapping(where: {destination_channel_id: {_eq: $destination_channel_id}}) {
            destination_channel_id
            unwrapped_chain {
              universal_chain_id
            }
            unwrapped_denom
          }
        }
      }
    `),
    {
      base_token: args.baseToken,
      source_chain_id: args.sourceChainId,
      // Not a bug! It is the dest of the original transfer, so now a source
      destination_channel_id: args.sourceChannelId
    }
  ).pipe(
    Effect.map(
      flow(
        Struct.get("v2_tokens"),
        Arr.head,
        Option.map(Struct.get("wrapping")),
        Option.flatMap(Arr.head),
        Option.map(Struct.get("unwrapped_denom"))
      )
    )
  )
