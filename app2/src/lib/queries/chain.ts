import { graphql } from "gql.tada"

export let chainsQueryDocument = graphql(`
  query Chains {
    v1_ibc_union_chains(where: {enabled: {_eq: true}}) {
      chain_id,
      display_name,
      addr_prefix
    }
  }
`)
