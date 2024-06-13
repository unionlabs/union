import { graphql } from "gql.tada"

export const chainsQueryDocument = graphql(/* GraphQL */ `query ChainsQuery @cached(ttl: 30) {
  v0_chains(where: {enabled: {_eq: true}}, order_by: {display_name: asc}) {
    display_name
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
    }
  }
}`)
