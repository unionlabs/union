import { raise } from "$lib/utilities/index.ts"
import { bytesToBech32Address } from "@union/client"
import { getCosmosChainBalances } from "./cosmos.ts"
import { createQueries } from "@tanstack/svelte-query"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { getBalancesFromAlchemy } from "./evm/alchemy.ts"
import { getBalancesFromRoutescan } from "./evm/routescan.ts"

export interface TokenBalance {
  name: string
  symbol: string
  address: string
  balance: bigint
  gasToken: boolean
  decimals: number
}

export function userBalancesQuery({
  chains,
  connected,
  userAddresses
}: {
  connected: boolean
  chains: Array<Chain>
  userAddresses: UserAddresses
}) {
  return createQueries({
    queries: chains.map(chain => ({
      // Using JSON.stringify to ensure queryKey updates when userAddr changes.
      queryKey: [
        "balances",
        chain.chain_id,
        userAddresses?.evm?.normalized,
        userAddresses?.cosmos?.normalized
      ],
      refetchInterval: 4_000,
      refetchOnWindowFocus: false,
      queryFn: async () => {
        if (chain.rpc_type === "evm" && userAddresses.evm && connected) {
          const rpc = chain.rpcs
            .filter(rpc => rpc.type === "alchemy" || rpc.type === "routescan")
            .at(0)
          if (!rpc) {
            raise(`No Alchemy or Routescan RPC available for chain ${chain.chain_id}`)
          }

          if (rpc.type === "alchemy") {
            return await getBalancesFromAlchemy({
              url: rpc.url,
              walletAddress: userAddresses.evm.canonical
            })
          }
          if (rpc.type === "routescan") {
            return await getBalancesFromRoutescan({
              url: rpc.url,
              walletAddress: userAddresses.evm.canonical
            })
          }
        }

        if (chain.rpc_type === "cosmos" && userAddresses.cosmos && connected) {
          const url = chain.rpcs.filter(rpc => rpc.type === "rest").at(0)?.url
          if (!url) raise(`No REST RPC available for chain ${chain.chain_id}`)

          const bech32_addr = bytesToBech32Address({
            bytes: userAddresses.cosmos.bytes,
            toPrefix: chain.addr_prefix
          })
          return getCosmosChainBalances({ url, walletAddress: bech32_addr })
        }

        return []
      }
    }))
  })
}
