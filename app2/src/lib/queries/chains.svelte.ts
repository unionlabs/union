import type { Environment } from "$lib/constants"
import { chains } from "$lib/stores/chains.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import { Chains } from "@unionlabs/sdk/schema"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"

export const chainsQuery = (environment: Environment) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_chains: Chains }),
    document: graphql(`
        query Chains($environment: String!) @cached(ttl: 60) {
            v2_chains {
                chain_id
                universal_chain_id
                minter_address_display
                display_name
                addr_prefix
                rpc_type
                testnet
                editions {
                    environment
                    name
                }
                features(where: { environment: { _eq: $environment } }) {
                    channel_list
                    connection_list
                    index_status
                    packet_list
                    transfer_submission
                    transfer_list
                }
                rpcs {
                    type
                    url
                }
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
    refetchInterval: "1 hour",
    writeData: data => {
      console.log({ chains: data })
      chains.data = data.pipe(Option.map(d => d.v2_chains))
    },
    writeError: error => {
      chains.error = error
    },
  })
