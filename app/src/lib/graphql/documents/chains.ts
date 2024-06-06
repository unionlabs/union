import { graphql } from "gql.tada"

export const chainsQueryDocument = graphql(/* GraphQL */ `query ChainsQuery {
  v0_chains {
    bech32_prefix
    chain_id
    display_name
    id
    rpc_type
    testnet
  }
}`)
