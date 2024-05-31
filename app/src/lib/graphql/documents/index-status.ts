import { graphql } from "gql.tada"

export const indexStatusQuery = graphql(/* GraphQL */ `
  query IndexStatusQuery {
    v0_index_status(order_by: { id: asc }) {
      chain_id
      display_name
      height
      id
      status
      timestamp
      tip_age_seconds
    }
  }
`)
