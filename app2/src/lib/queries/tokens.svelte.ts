import { Effect, Option, Schema } from "effect"
import type { UniversalChainId } from "$lib/schema/chain"
import { Token, TokenRawDenom } from "$lib/schema/token"
import { createQueryGraphql } from "$lib/utils/queries"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { graphql } from "gql.tada"

export const tokensQuery = (universalChainId: UniversalChainId) =>
  Effect.gen(function* () {
    yield* Effect.log(`zkgm starting token fetcher for ${universalChainId}`)
    const response = yield* createQueryGraphql({
      schema: Schema.Struct({ v1_ibc_union_tokens: Schema.Array(Token) }),
      document: graphql(`
        query TokensForChain($universal_chain_id: String!) @cached(ttl: 60) {
          v1_ibc_union_tokens(where: { chain: { universal_chain_id: { _eq: $universal_chain_id } } }) {
            denom
            cw20 {
              cw20_token_address
            }
            representations {
              name
              symbol
              decimals
              sources {        
                update_timestamp
                source {
                  name
                  logo_uri
                  source_uri
                }
                wrapping {
                  destination_channel_id
                  unwrapped_chain {
                    universal_chain_id
                  }
                  unwrapped_denom
                }
              }
            }
            wrapping {
              unwrapped_chain {
                universal_chain_id
              }
              wrapped_chain {
                universal_chain_id
              }
              destination_channel_id
              unwrapped_denom
            }
          }
        }
      `),
      variables: { universal_chain_id: universalChainId },
      refetchInterval: "10 minutes",
      writeData: data => {
        Effect.runSync(Effect.log(`storing new tokens for ${universalChainId}`))
        tokensStore.setData(
          universalChainId,
          // Can be removed when this invalid denom is removed from hubble
          data.pipe(
            Option.map(d =>
              d.v1_ibc_union_tokens.filter(
                token =>
                  token.denom !== TokenRawDenom.make("0x0000000000000000000000000000000000000000")
              )
            )
          )
        )
      },
      writeError: error => {
        Effect.runSync(Effect.log(`storing new tokens error for ${universalChainId}`))
        tokensStore.setError(universalChainId, error)
      }
    })
    return response
  })

export const tokenWrappingQuery = graphql(/* GraphQL */ `
    query QueryTokenWrapping($source_chain_id: String!, $base_token: String!, $destination_channel_id: Int!) {
        v1_ibc_union_tokens(where: {_and: {chain: {chain_id: {_eq: $source_chain_id}}, denom: {_eq: $base_token}, wrapping: {_and: {index: {_eq: 0}, destination_channel_id: {_eq: $destination_channel_id}}}}}) {
            wrapping {
                unwrapped_address_hex
            }
        }
    }
`)
