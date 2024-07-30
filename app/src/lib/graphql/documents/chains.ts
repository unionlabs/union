import { graphql } from "gql.tada"

export const chainsQueryDocument = graphql(/* GraphQL */ `query ChainsQuery @cached(ttl: 30) {
  v0_chains(where: {enabled: {_eq: true}} order_by: {display_name: asc}) {
    display_name
    testnet
    chain_id
    enabled
    id
    rpc_type
    addr_prefix
    rpcs(where: {enabled: {_eq: true}}) {
      url
      type
    }
    ucs1_configurations {
      channel_id
      contract_address
      destination_chain {
        chain_id
      }
      forward {
        channel_id
        destination_chain {
          chain_id
        }
        port
      }
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
  }
}`)
