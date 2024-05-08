import { graphql } from "gql.tada"
import { URLS } from "$lib/constants"
import { request } from "graphql-request"
import { createQuery } from "@tanstack/svelte-query"
import type { ChainId } from "$/lib/constants/assets"

/**
 * TODO:
 * - [ ] Update the GraphQL query to be chain agnostic and receive the chain as a parameter
 */

export function balanceQuery<TChain extends ChainId>({
  chain,
  asset,
  address,
  refetchInterval = 4_000
}: {
  chain: TChain
  address: string
  asset: string
  refetchInterval?: number
}) {
  return createQuery({
    queryKey: ["balance", chain, asset, address],
    queryFn: async () =>
      request(
        URLS.GRAPHQL,
        // TODO: Update the query once REST API codegen is done
        graphql(/* GraphQL */ `
        query userBalances($address: String!) {
          __typename
        }`),
        { address }
      ),
    enabled: !!address
  })
}
