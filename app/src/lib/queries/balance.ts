import * as v from "valibot"
import { getEvmTokensInfo } from "./token-info.ts"
import { createQueries } from "@tanstack/svelte-query"
import { raise } from "$lib/utilities/index.ts"
import { rawToBech32 } from "$lib/utilities/address.ts"
import type { Address } from "viem"
import type { Chain, UserAddresses } from "$lib/types.ts"

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

const cosmosBalancesResponseSchema = v.object({
  balances: v.array(
    v.object({
      denom: v.string(),
      amount: v.string()
    })
  )
})


export type EvmBalances = v.InferOutput<typeof evmBalancesResponseSchema>

export function userBalancesQuery({
  userAddr,
  chains
}: {
  userAddr: UserAddresses
  chains: Array<Chain>
}) {
  return createQueries({
    queries: chains.map(chain => ({
      // note: we assume each chain only has one userAddr. this might change later
      queryKey: ["balances", chain.chain_id, userAddr.evm.normalized],
      refetchOnWindowFocus: false,
      refetchInterval: 2_000,
      queryFn: async () => {

        if (chain.rpc_type === "evm") {

          let json: undefined | unknown


          // TODO: support quicknode
          const alchemy_rpcs = chain.rpcs.filter(rpc => rpc.type === "alchemy")
          if (alchemy_rpcs.length === 0) raise(`no alchemy rpc available for chain ${chain.chain_id}`)

          const alchemyUrl = alchemy_rpcs[0].url

          try {
            const response = await fetch(`https://${alchemyUrl}`, {
              method: "POST",
              body: JSON.stringify({
                id: 1,
                jsonrpc: "2.0",
                method: "alchemy_getTokenBalances",
                params: [userAddr.evm.canonical, "erc20"]
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
        
        if (chain.rpc_type === "cosmos") {
          const bech32_addr = rawToBech32(chain.addr_prefix, userAddr.cosmos.bytes);

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

        raise(`chain has unsupported rpc type ${chain.rpc_type}`);
      }
    }))
  })
}
