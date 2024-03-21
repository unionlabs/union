import { graphql } from "gql.tada"
import { URLS } from "$/lib/constants"
import { request } from "graphql-request"
import { createQuery } from "@tanstack/svelte-query"
import type { Chain, ChainAsset } from "$/lib/constants/assets"

/**
 * TODO:
 * - [ ] Update the GraphQL query to be chain agnostic and receive the chain as a parameter
 */

export function balanceQuery<TChain extends Chain>({
  chain,
  asset,
  address
}: { chain: TChain; address: string; asset: ChainAsset<TChain> }) {
  return createQuery({
    queryKey: ["balance", chain, asset, address],
    queryFn: async () =>
      request(
        URLS.GRAPHQL,
        graphql(/* GraphQL */ `
        query userBalances($address: String!) {
          cosmosBankV1Beta1AllBalances(address: $address) {
            balances { amount denom }
          }
        }`),
        { address }
      ),
    enabled: !!address
  })
}
