import { raise } from "$lib/utilities/index.ts"
import { createQueries } from "@tanstack/svelte-query"
import { rawToBech32 } from "$lib/utilities/address.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { getEvmChainBalances } from "$lib/queries/balance/evm/index.ts"
import { getCosmosChainBalances } from "$lib/queries/balance/cosmos.ts"

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
          const url = chain.rpcs.filter(rpc => rpc.type === "alchemy")?.at(0)?.url
          if (!url) raise(`no alchemy rpc available for chain ${chain.chain_id}`)
          return getEvmChainBalances({ url, walletAddress: userAddr.evm.canonical })
        }

        if (chain.rpc_type === "cosmos") {
          const restUrl = chain.rpcs.filter(rpc => rpc.type === "rest").at(0)?.url
          if (!restUrl) raise(`no rest rpc available for chain ${chain.chain_id}`)

          const bech32_addr = rawToBech32(chain.addr_prefix, userAddr.cosmos.bytes)
          return getCosmosChainBalances({ url: `https://${restUrl}`, walletAddress: bech32_addr })
        }

        return []
      }
    }))
  })
}
