import * as v from "valibot"
import { KEY } from "$lib/constants/keys.ts"
import { CHAIN_URLS } from "$lib/constants";
import { formatUnits, type Address } from "viem"
import { getEvmTokensInfo } from "./token-info.ts"
import { createQuery } from "@tanstack/svelte-query"
import type { ChainId } from "$/lib/constants/assets.ts"
import { isValidEvmAddress } from "$lib/wallet/utilities/validate"
import { isValidCosmosAddress } from "$lib/wallet/utilities/validate";

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
    // TODO: Update the query once REST API codegen is done
    queryFn: () => {
      throw new Error("Not implemented")
    },
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
    queryKey: ["balances", chainId, address],
    enabled: isValidEvmAddress(address),
    refetchOnWindowFocus: false,
    refetchInterval: 1_000,
    queryFn: async () => {
      const assetsToCheck =
        "contractAddresses" in restParams && Array.isArray(restParams.contractAddresses)
          ? restParams.contractAddresses // if contractAddresses is an array, use it
          : "tokenSpecification" in restParams &&
              ["erc20", "DEFAULT_TOKENS"].includes(restParams.tokenSpecification)
            ? restParams.tokenSpecification // if tokenSpecification is a string, use it
            : "DEFAULT_TOKENS"
      console.log(address, assetsToCheck)
      const response = await fetch(`https://eth-sepolia.g.alchemy.com/v2/${KEY.RPC.ALCHEMY}`, {
        method: "POST",
        body: JSON.stringify({
          id: 1,
          jsonrpc: "2.0",
          method: "alchemy_getTokenBalances",
          params: [address, assetsToCheck]
        })
      })
      const result = v.safeParse(evmBalancesResponseSchema, await response.json())
      if (!result.success) throw new Error(`Error parsing result ${JSON.stringify(result.issues)}`);

      const tokensInfo = await getEvmTokensInfo(
        result.output.result.tokenBalances.map(({ contractAddress }) => contractAddress)
      )
      return tokensInfo.map((token, index) => ({
        ...token,
        balance: BigInt(result.output.result.tokenBalances[index].tokenBalance)
      }))
    }
  })
}

export function cosmosBalancesQuery({
  address,
  chainId
}: {
  address: string
  chainId: string
}) {
  return createQuery({
    queryKey: ["balances", chainId, address],
    enabled: isValidCosmosAddress(address),
    refetchOnWindowFocus: false,
    queryFn: async () => {
        const restUrl = CHAIN_URLS[chainId].REST
        const response = await fetch(
            `${restUrl}/cosmos/bank/v1beta1/balances/${address}`,
        {});
        if (!response.ok) return new Error("invalid response");
        return (await response.json()).balances.map((x) => {
            return {
                address: x.denom,
                symbol: x.denom,
                balance: x.amount,
                decimals: 0
            }
        })
    }
  })
}
