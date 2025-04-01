import { Chains } from "@unionlabs/sdk/schema"
import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { chains } from "$lib/stores/chains.svelte"
import type { Environment } from "$lib/constants"

export let chainsQuery = (environment: Environment) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_chains: Chains }),
    document: graphql(`
    query Chains($environment: String!) @cached(ttl: 60) {
      v2_chains {
        chain_id,
        universal_chain_id,
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
        },
        rpcs {
          type
          url
        },
        explorers {
          address_url
          block_url
          description
          display_name
          home_url
          name
          tx_url
        }
      }
    }
  `),
    variables: { environment },
    refetchInterval: "60 seconds",
    writeData: data => {
      chains.data = data.pipe(Option.map(d => d.v2_chains))
    },
    writeError: error => {
      chains.error = error
    }
  })
