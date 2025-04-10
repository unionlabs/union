import { TokenRawDenom } from "../schema/token.js";
import { UniversalChainId } from "../schema/chain.js";
import { fetchDecodeGraphql } from "../utils/graphql-query.js";
import { Schema } from "effect";
import { graphql } from "gql.tada"

export const tokenWrappingQuery = ({
  denom,
  universal_chain_id
}: { denom: TokenRawDenom; universal_chain_id: UniversalChainId }) =>
  fetchDecodeGraphql(
    Schema.Struct({
      v2_tokens: Schema.Array(
        Schema.Struct({
          wrapping: Schema.Array(Schema.Struct({ unwrapped_denom: TokenRawDenom }))
        })
      )
    }),
    graphql(/* GraphQL */ `
        query TokenWrapping($denom: String!, $universal_chain_id: String!) {
          v2_tokens(args: {
            p_denom: $denom,
            p_universal_chain_id: $universal_chain_id
          }) {
            wrapping {
              unwrapped_denom
            }
          }
        }
    `),
    {
      denom,
      universal_chain_id
    }
  )
