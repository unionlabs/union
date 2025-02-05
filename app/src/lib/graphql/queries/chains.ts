import { graphql } from "gql.tada"

export const chainsQueryDocument =
  graphql(/* GraphQL */ `query ChainsQuery($environment: String!) @cached(ttl: 30) {
  v1_ibc_union_chains(order_by: {display_name: asc}) {
    display_name
    testnet
    chain_id
    features(where: {environment: {_eq: $environment}}) {
      channel_list
      connection_list
      environment
      index_status
      packet_list
      transfer_list
      transfer_submission
    }    
    enabled
    rpc_type
    addr_prefix
    enabled
    enabled_staging
    rpcs(where: {enabled: {_eq: true}}) {
      url
      type
    }
    explorers {
      tx_url
      block_url
      address_url
    }
    assets {
      denom
      display_symbol
      display_name
      decimals
      faucets(where: { enabled: {_eq: true}}) {
        url
        display_name
      }
      gas_token
    }
    tokens {
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
}`)
