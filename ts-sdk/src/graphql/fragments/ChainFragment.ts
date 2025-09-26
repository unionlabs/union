import { graphql } from "gql.tada"

export const ChainFragment = graphql(`
fragment ChainFragment on v2_chain_type {
  chain_id
  universal_chain_id
  minter_address_display
  display_name
  addr_prefix
  rpc_type
  testnet
  editions {
      environment
      name
  }
  features(where: { environment: { _eq: "PRODUCTION" } }) {
      channel_list
      connection_list
      index_status
      packet_list
      transfer_submission
      transfer_list
  }
  rpcs {
      type
      url
  }
  explorers {
      address_url
      block_url
      description
      display_name
      home_url
      name
      tx_url
  }
}
`)
