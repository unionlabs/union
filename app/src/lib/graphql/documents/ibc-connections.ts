import { graphql } from "gql.tada"

export const ibcConnectionMapQuery = graphql(/* graphql */ `
  query IBCConnectionMapQuery($limit: Int = 500, $status: String = "CONFIRM") @cached(ttl: 30) {
    data: v0_connection_map(limit: $limit, where: { status: { _eq: $status }}) {
      from_chain_id
      from_client_id
      from_connection_id
      from_id
      status
      to_id
      to_chain_id
      to_client_id
      to_connection_id
    }
  }
`)
