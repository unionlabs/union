import { graphql } from "gql.tada"

export const ibcClientMapQuery = graphql(/* graphql */ `
  query IBCClientMapQuery {
    v0_clients {
      chain_id
      client_id
      counterparty_chain_id
    }
  }
`)
