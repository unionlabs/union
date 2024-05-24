import { graphql } from "gql.tada"

export const ibcConnectionMapQuery = graphql(/* graphql */ `
  query IBCConnectionMapQuery {
    data: v0_connection_map(where: { status: { _eq: "CONFIRM" }}) {
      from_chain_id
      from_client_id
      from_connection_id
      from_id
      to_id
      status
      to_chain_id
      to_client_id
      to_connection_id
    }
  }
`)
