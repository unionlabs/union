import { graphql } from "gql.tada"
import { URLS } from "$/lib/constants"
import { request } from "graphql-request"
import { createQuery } from "@tanstack/svelte-query"
import type { Chain, ChainAsset } from "$/lib/constants/assets"

/**
 * TODO:
 * - [x] Add Union transfers query
 * - [ ] Add Sepolia transfers query
 */

export function transfersQuery<TChain extends Chain>({
  chain,
  address
}: { chain: TChain; address: string }) {
  return createQuery({
    queryKey: ["transfers", chain, address],
    queryFn: async () =>
      request(
        URLS.GRAPHQL,
        graphql(/* GraphQL */ `
          query userTransfers($address: String!) {
            v0_wasm_ibc_transfers(limit: 10) {
              sender
              receiver
            }
          }
        `),
        { address }
      ),
    enabled: !!address
  })
}
