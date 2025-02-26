import { Chains } from "$lib/schema/chain"
import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { chains } from "$lib/stores/chains.svelte"

export let chainsQuery = createQueryGraphql({
  schema: Schema.Struct({ v1_ibc_union_chains: Chains }),
  document: graphql(`
    query Chains {
      v1_ibc_union_chains(where: {enabled: {_eq: true}}) {
        chain_id,
        display_name,
        addr_prefix
      }
    }
  `),
  refetchInterval: "5 seconds",
  writeData: data => {
    chains.data = data.pipe(Option.map(d => d.v1_ibc_union_chains))
  },
  writeError: error => {
    chains.error = error
  }
})
