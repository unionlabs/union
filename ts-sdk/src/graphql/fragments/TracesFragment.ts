import { graphql } from "gql.tada"

export const TracesFragment = graphql(`
fragment TracesFragment on v2_traces_type {
  type
  height
  block_hash
  timestamp
  transaction_hash
  chain {
    universal_chain_id
  }
}
`)
