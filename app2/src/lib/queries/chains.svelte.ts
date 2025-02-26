import { Chains } from "$lib/schema/chain"
import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { chains } from "$lib/stores/chains.svelte"
import type { Environment } from "$lib/constants"

export let chainsQuery = (environment: Environment) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_chains: Chains }),
    document: graphql(`
    query Chains($environment: String!) @cached(ttl: 60) {
      v1_ibc_union_chains(where: {enabled: {_eq: true}}) {
        chain_id,
        display_name,
        addr_prefix,
        rpc_type,
        testnet,
        features(where: {environment: {_eq: $environment}}) {
          channel_list
          connection_list,
          index_status,
          packet_list,
          transfer_submission,
          transfer_list
        }
      }
    }
  `),
    variables: { environment },
    refetchInterval: "5 seconds",
    writeData: data => {
      chains.data = data.pipe(Option.map(d => d.v1_ibc_union_chains))
    },
    writeError: error => {
      chains.error = error
    }
  })
