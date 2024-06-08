import { graphql } from "gql.tada"

export const ibcClientMapQuery = graphql(/* graphql */ `
  query IBCClientMapQuery($limit: Int = 600) @cached(ttl: 30) {
    data: v0_clients(limit: $limit) {
      chain_id
      client_id
      counterparty_chain_id
    }
  }
`)
