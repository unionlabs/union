import { isAddress, type Address } from "viem"
import { raise } from "$lib/utilities/index.ts"
import { getCosmosChainBalances } from "./cosmos.ts"
import { createQueries } from "@tanstack/svelte-query"
import { erc20ReadMulticall } from "./evm/multicall.ts"
import { rawToBech32 } from "$lib/utilities/address.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { getBalancesFromAlchemy } from "./evm/alchemy.ts"
import { getBalancesFromRoutescan } from "./evm/routescan.ts"

export function userBalancesQuery({
  userAddr,
  chains,
  connected
}: {
  userAddr: UserAddresses
  chains: Array<Chain>
  connected: boolean
}) {
  return createQueries({
    queries: chains.map(chain => ({
      // Using JSON.stringify to ensure queryKey updates when userAddr changes.
      queryKey: [
        "balances",
        chain.chain_id,
        userAddr?.evm?.normalized,
        userAddr?.cosmos?.normalized
      ],
      refetchOnWindowFocus: false,
      refetchInterval: 4_000,
      queryFn: async () => {
        if (chain.rpc_type === "evm" && userAddr.evm && connected) {
          const rpc = chain.rpcs
            .filter(rpc => rpc.type === "alchemy" || rpc.type === "routescan")
            .at(0)

          if (rpc?.type === "alchemy") {
            return await getBalancesFromAlchemy({
              url: rpc.url,
              walletAddress: userAddr.evm.canonical
            })
          }
          if (rpc?.type === "routescan") {
            return await getBalancesFromRoutescan({
              url: rpc.url,
              walletAddress: userAddr.evm.canonical
            })
          }

          const tokenList = chain.assets.filter(asset => isAddress(asset.denom))
          const multicallResults = await erc20ReadMulticall({
            chainId: chain.chain_id,
            functionNames: ["balanceOf"],
            address: userAddr.evm.canonical,
            contractAddresses: tokenList.map(asset => asset.denom) as Array<Address>
          })

          return multicallResults
            .map((result, index) => ({
              balance: result.balance,
              address: tokenList[index].denom,
              name: tokenList[index].display_name,
              symbol: tokenList[index].display_symbol,
              gasToken: tokenList[index].gas_token ?? false
            }))
            .filter(result => !!result?.balance && BigInt(result.balance) > 0n)
        }

        if (chain.rpc_type === "cosmos" && userAddr.cosmos && connected) {
          const url = chain.rpcs.filter(rpc => rpc.type === "rest").at(0)?.url
          if (!url) raise(`No REST RPC available for chain ${chain.chain_id}`)

          const bech32_addr = rawToBech32(chain.addr_prefix, userAddr.cosmos.bytes)
          return getCosmosChainBalances({ url, walletAddress: bech32_addr })
        }

        return []
      }
    }))
  })
}
