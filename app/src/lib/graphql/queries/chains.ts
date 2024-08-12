import { graphql } from "../index.ts"
import type { ExtractData } from "../types.ts"
import type { NonNullableFields } from "$lib/utilities/types.ts"

export const chainsQueryDocument = graphql(/* GraphQL */ `
  query ChainsQuery(
    $includeRpcs: Boolean = true,
    $includeContracts: Boolean = true,
    $includeEndpoints: Boolean = true
  ) @cached(ttl: 30) {
    data: v0_chains(order_by: { display_name: asc }) {
      id
      enabled
      testnet
      chain_id
      rpc_type
      logo_uri
      addr_prefix
      display_name
      enabled_staging
      rpcs(where: { enabled: { _eq: true }}) @include(if: $includeRpcs) {
        url
        type
      }
      ucs1_configurations @include(if: $includeContracts) {
        channel_id
        contract_address
        source_chain {
          id
          testnet
          enabled
          chain_id
          logo_uri
          addr_prefix
          display_name
        }
        destination_chain {
          id
          testnet
          enabled
          chain_id
          logo_uri
          addr_prefix
          display_name
        }
        forward {
          port
          channel_id
          connection_id
          contract_address
          destination_chain {
            chain_id
            addr_prefix
          }
        }
      }
      explorers {
        tx_url
        block_url
        address_url
      }
      assets @include(if: $includeEndpoints) {
        denom
        decimals
        gas_token
        display_name
        display_symbol
        faucets(where: { enabled: { _eq: true }}) {
          url
          display_name
        }
      }
    }
  }
`)

export type ChainsQueryResult = NonNullableFields<
  ExtractData<typeof chainsQueryDocument>["data"][number]
>
