import * as v from "valibot"
import { KEY } from "$lib/constants/keys.ts"
import { getEvmTokensInfo } from "./token-info.ts"
import { createQueries, createQuery } from "@tanstack/svelte-query"
import type { ChainId } from "$/lib/constants/assets.ts"
import { isValidEvmAddress } from "$lib/wallet/utilities/validate"
import { raise } from "$lib/utilities/index.ts"
import { rawToBech32 } from "$lib/utilities/address.ts"
import { DecodeLogDataMismatch, type Address } from "viem"

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
    address: v.pipe(v.string(), v.length(42)),
    tokenBalances: v.array(
      v.object({
        contractAddress: v.pipe(v.string(), v.length(42)),
        tokenBalance: v.string()
      })
    )
  })
})

export type EvmBalances = v.InferOutput<typeof evmBalancesResponseSchema>

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
    refetchInterval: 200_000,
    queryFn: async () => {
      DecodeLogDataMismatch
      const assetsToCheck =
        "contractAddresses" in restParams && Array.isArray(restParams.contractAddresses)
          ? restParams.contractAddresses // if contractAddresses is an array, use it
          : "tokenSpecification" in restParams &&
              ["erc20", "DEFAULT_TOKENS"].includes(restParams.tokenSpecification)
            ? restParams.tokenSpecification // if tokenSpecification is a string, use it
            : "DEFAULT_TOKENS"

      let json: undefined | unknown

      try {
        const response = await fetch(`https://eth-sepolia.g.alchemy.com/v2/${KEY.RPC.ALCHEMY}`, {
          method: "POST",
          body: JSON.stringify({
            id: 1,
            jsonrpc: "2.0",
            method: "alchemy_getTokenBalances",
            params: [address, assetsToCheck]
          })
        })
        if (!response.ok) raise("error fetching from alchemy: non-200 status")
        json = await response.json()
      } catch (err) {
        if (err instanceof Error) {
          raise(`error fetching from alchemy: ${err.message}`)
        }
        raise(`unknown error while fetching from alchemy: ${JSON.stringify(err)}`)
      }
      const result = v.safeParse(evmBalancesResponseSchema, json)

      if (!result.success) raise(`error parsing result ${JSON.stringify(result.issues)}`)

      const tokensInfo = await getEvmTokensInfo(
        result.output.result.tokenBalances.map(({ contractAddress }) => contractAddress)
      )
      return tokensInfo.map((token, index) => ({
        ...token,
        balance: BigInt(result.output.result.tokenBalances[index].tokenBalance),
        address: token.address as Address
      }))
    }
  })
}

const cosmosBalancesResponseSchema = v.object({
  balances: v.array(
    v.object({
      denom: v.string(),
      amount: v.string()
    })
  )
})

export function cosmosBalancesQuery({
  address,
  chains
}: {
  address: Uint8Array
  chains: Array<{
    chain_id: string
    addr_prefix: string
    rpcs: Array<{ url: string; type: string }>
  }>
}) {
  console.log(address)
  return createQueries({
    queries: chains.map(chain => {
      const bech32_addr = rawToBech32(chain.addr_prefix, address)
      console.log(bech32_addr)

      return {
        queryKey: ["balances", chain.chain_id, bech32_addr],
        enabled: true,
        refetchInterval: 200_000,
        refetchOnWindowFocus: false,
        queryFn: async () => {
          let json: undefined | unknown
          const rest_rpcs = chain.rpcs.filter(rpc => rpc.type === "rest")
          if (rest_rpcs.length === 0) raise(`no rest rpc available for chain ${chain.chain_id}`)

          const restUrl = rest_rpcs[0].url

          try {
            const response = await fetch(
              `https://${restUrl}/cosmos/bank/v1beta1/balances/${bech32_addr}`
            )

            if (!response.ok) throw new Error("invalid response")

            json = await response.json()
          } catch (err) {
            if (err instanceof Error) {
              raise(`error fetching balances from /cosmos/bank: ${err.message}`)
            }
            raise(`unknown error while fetching from /cosmos/bank: ${JSON.stringify(err)}`)
          }

          const result = v.safeParse(cosmosBalancesResponseSchema, json)

          if (!result.success) raise(`error parsing result ${JSON.stringify(result.issues)}`)
          return result.output.balances.map(x => ({
            address: x.denom,
            symbol: x.denom,
            balance: x.amount,
            decimals: 0
          }))
        }
      }
    })
  })
}
