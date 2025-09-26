import { graphql } from "gql.tada"

export const TokenFragment = graphql(`
fragment TokenFragment on v2_token_meta {
  rank
  denom
  bucket {
      capacity
      refill_rate
  }
  representations {
    logo_uri
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
`)
