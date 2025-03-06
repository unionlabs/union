import { Effect, Option, Schema } from "effect"
import type { UniversalChainId } from "$lib/schema/chain"
import { Token } from "$lib/schema/token"
import { createQueryGraphql } from "$lib/utils/queries"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { graphql } from "gql.tada"

export const tokensQuery = (universalChainId: UniversalChainId) =>
  Effect.gen(function* () {
    yield* Effect.log(`starting token fetcher for ${universalChainId}`)
    const response = yield* createQueryGraphql({
      schema: Schema.Struct({ v1_ibc_union_tokens: Schema.Array(Token) }),
      document: graphql(`
        query TokensForChain($universal_chain_id: String!) @cached(ttl: 60) {
          v1_ibc_union_tokens(where: { chain: { universal_chain_id: { _eq: $universal_chain_id } } }) {
            denom
            cw20 {
              cw20_token_address
            }
            chain {
              chain_id
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
                    chain_id
                  }
                  unwrapped_denom
                }
              }
            }
            wrapping {
              unwrapped_chain {
                chain_id
              }
              wrapped_chain {
                chain_id
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
        tokensStore.setData(universalChainId, data.pipe(Option.map(d => d.v1_ibc_union_tokens)))
      },
      writeError: error => {
        Effect.runSync(Effect.log(`storing new tokens error for ${universalChainId}`))
        tokensStore.setError(universalChainId, error)
      }
    })
    return response
  })
