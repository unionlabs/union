import { graphql } from "gql.tada"

export const tokensQueryDocument = graphql(/* GraphQL */ `
query TokensQuery {
  v1_ibc_union_tokens {
    denom
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
      wrapped_chain {
        chain_id
      }
      destination_channel_id
      unwrapped_denom
    }
  }
}
`)
