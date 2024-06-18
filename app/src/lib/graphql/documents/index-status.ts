import { graphql } from "gql.tada"

export const indexStatusQuery = graphql(/* GraphQL */ `
  query IndexStatusQuery {
    chains: v0_chains(where: {enabled:{_eq:true}}) {
      chain_id
    }
    statuses: v0_index_status(order_by: {id: asc}) {
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
