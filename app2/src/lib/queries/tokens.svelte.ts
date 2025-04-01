import { Effect, Option, Schema } from "effect"
import type { UniversalChainId } from "@unionlabs/sdk/schema"
import { TokenRawDenom, Tokens } from "@unionlabs/sdk/schema"
import { isTokenBlacklisted } from "$lib/constants/tokens"
import { createQueryGraphql, fetchDecodeGraphql } from "$lib/utils/queries"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { graphql } from "gql.tada"

export const tokensQuery = (universalChainId: UniversalChainId) =>
  Effect.gen(function* () {
    yield* Effect.log(`zkgm starting token fetcher for ${universalChainId}`)
    const response = yield* createQueryGraphql({
      schema: Schema.Struct({ v2_tokens: Tokens }),
      document: graphql(`
        query TokensForChain($universal_chain_id: String!) @cached(ttl: 60) {
          v2_tokens(args: { p_universal_chain_id: $universal_chain_id }, order_by: {rank: asc_nulls_last}) {
            rank
            denom
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
          // Filter out blacklisted tokens
          data.pipe(Option.map(d => d.v2_tokens.filter(token => !isTokenBlacklisted(token.denom))))
        )
      },
      writeError: error => {
        Effect.runSync(Effect.log(`storing new tokens error for ${universalChainId}`))
        tokensStore.setError(universalChainId, error)
      }
    })
    return response
  })

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
        query($denom: String!, $universal_chain_id: String!) {
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
