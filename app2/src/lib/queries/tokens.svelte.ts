import { Effect, Option, Schema } from "effect"
import type { ChainId, UniversalChainId } from "$lib/schema/chain"
import { TokenRawDenom, Tokens } from "$lib/schema/token"
import { isTokenBlacklisted } from "$lib/constants/tokens"
import { createQueryGraphql, fetchDecodeGraphql } from "$lib/utils/queries"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { graphql } from "gql.tada"
import type { ChannelId } from "$lib/schema/channel"

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
  base_token,
  destination_channel_id,
  source_chain_id
}: { base_token: TokenRawDenom; destination_channel_id: ChannelId; source_chain_id: ChainId }) =>
  fetchDecodeGraphql(
    Schema.Struct({
      v1_ibc_union_tokens: Schema.Array(
        Schema.Struct({
          wrapping: Schema.Array(Schema.Struct({ unwrapped_address_hex: TokenRawDenom }))
        })
      )
    }),
    graphql(/* GraphQL */ `
        query QueryTokenWrapping($source_chain_id: String!, $base_token: String!, $destination_channel_id: Int!) {
            v1_ibc_union_tokens(where: {_and: {chain: {chain_id: {_eq: $source_chain_id}}, denom: {_eq: $base_token}, wrapping: {_and: {index: {_eq: 0}, destination_channel_id: {_eq: $destination_channel_id}}}}}) {
                wrapping {
                    unwrapped_address_hex
                }
            }
        }
    `),
    {
      base_token,
      destination_channel_id,
      source_chain_id
    }
  )
