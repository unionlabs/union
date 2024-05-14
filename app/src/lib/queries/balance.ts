import * as v from "valibot"
import { graphql } from "gql.tada"
import { URLS } from "$lib/constants"
import { request } from "graphql-request"
import { API_KEY } from "$lib/constants/keys"
import { formatUnits, type Address } from "viem"
import { getEvmTokensInfo } from "./token-info.ts"
import { createQuery } from "@tanstack/svelte-query"
import type { ChainId } from "$/lib/constants/assets.ts"
import { isValidEvmAddress } from "$lib/wallet/utilities/validate"

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

const evmBalancesResponseSchema = v.object({
  jsonrpc: v.string(),
  id: v.number(),
  result: v.object({
    address: v.string([v.length(42)]),
    tokenBalances: v.array(
      v.object({
        contractAddress: v.string([v.length(42)]),
        tokenBalance: v.string()
      })
    )
  })
})

export type EvmBalances = v.Output<typeof evmBalancesResponseSchema>

/**
 * @docs https://docs.alchemy.com/reference/alchemy-gettokenbalances
 * @note the parameters here match the API parameters 1:1. See docs
 */
export function evmBalancesQuery({
  address,
  chainId,
  ...restParams
}: {
  address: Address
  chainId: string
} & ({ contractAddresses: Array<string> } | { tokenSpecification: "erc20" | "DEFAULT_TOKENS" })) {
  return createQuery({
    queryKey: [address, "balances", chainId],
    enabled: isValidEvmAddress(address),
    refetchOnWindowFocus: false,
    queryFn: async () => {
      const assetsToCheck =
        "contractAddresses" in restParams && Array.isArray(restParams.contractAddresses)
          ? restParams.contractAddresses // if contractAddresses is an array, use it
          : "tokenSpecification" in restParams &&
              ["erc20", "DEFAULT_TOKENS"].includes(restParams.tokenSpecification)
            ? restParams.tokenSpecification // if tokenSpecification is a string, use it
            : "DEFAULT_TOKENS"
      console.log(address, assetsToCheck)
      const response = await fetch(`https://eth-sepolia.g.alchemy.com/v2/${API_KEY.ALCHEMY}`, {
        method: "POST",
        body: JSON.stringify({
          id: 1,
          jsonrpc: "2.0",
          method: "alchemy_getTokenBalances",
          params: [address, assetsToCheck]
        })
      })
      const result = v.safeParse(evmBalancesResponseSchema, await response.json())
      if (!result.success) return null

      const tokensInfo = await getEvmTokensInfo(
        result.output.result.tokenBalances.map(({ contractAddress }) => contractAddress)
      )
      return tokensInfo.map((token, index) => ({
        ...token,
        balance: formatUnits(
          BigInt(result.output.result.tokenBalances[index].tokenBalance),
          token.decimals
        )
      }))
    }
  })
}
