import type { TokenRawDenom } from "../schema/token.js"
import type { UniversalChainId } from "../schema/chain.js"
import { fetchDecodeGraphql } from "../utils/graphql-query.js"
import { Schema } from "effect"
import { graphql } from "gql.tada"

export const tokenWrappingQuery = ({
  base_token: TokenRawDenom,
  source_chain_id: UniversalChainId
}: { base_token: TokenRawDenom; source_chain_id: UniversalChainId }) =>
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
      base_token,
      source_chain_id
    }
  )
