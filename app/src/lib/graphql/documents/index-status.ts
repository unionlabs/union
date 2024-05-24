import { graphql } from "gql.tada"

export const indexStatusQuery = graphql(/* GraphQL */ `
  query IndexStatusQuery {
    data: v0_index_status(order_by: { id: asc }) {
      id
      chain_id
      display_name
      height
      time
    }
  }
`)

export const indexStatusSubscription = graphql(/* GraphQL */ `
  subscription IndexStatusSubscription {
    data: v0_index_status(order_by: { id: asc }) {
      id
      chain_id
      display_name
      height
      time
    }
  }
`)
