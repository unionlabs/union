import { raise } from "$lib/utilities/index.ts"
import { createQueries } from "@tanstack/svelte-query"
import { rawToBech32 } from "$lib/utilities/address.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { getCosmosChainBalances } from "$lib/queries/balance/cosmos.ts"
import { getBalancesFromAlchemy } from "./evm/alchemy"
import { getBalancesFromRoutescan } from "./evm/routescan"

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
      refetchInterval: 4_000,
      queryFn: async () => {
        if (chain.rpc_type === "evm") {
          const rpc = chain.rpcs.filter(rpc => rpc.type === "alchemy" || rpc.type === "routescan").at(0);
          if (rpc === undefined) raise(`no alchemy or routescan rpc available for chain ${chain.chain_id}`)

          if (rpc.type === "alchemy") {
            return getBalancesFromAlchemy({ url: rpc.url, walletAddress: userAddr.evm.canonical })
          }
          if (rpc.type === "routescan") {
            return getBalancesFromRoutescan({ url: rpc.url, walletAddress: userAddr.evm.canonical })
          }
        }

        if (chain.rpc_type === "cosmos") {
          const url = chain.rpcs.filter(rpc => rpc.type === "rest").at(0)?.url
          if (!url) raise(`no rest rpc available for chain ${chain.chain_id}`)

          const bech32_addr = rawToBech32(chain.addr_prefix, userAddr.cosmos.bytes)
          return getCosmosChainBalances({ url, walletAddress: bech32_addr })
        }

        return []
      }
    }))
  })
}
