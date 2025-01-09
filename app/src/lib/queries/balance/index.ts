import { isAddress, type Address } from "viem"
import { bech32ToBech32Address } from "@unionlabs/client"
import type { Chain, UserAddresses, ChainAsset } from "$lib/types"
import { erc20ReadMulticall } from "./evm/multicall.ts"
import { getCosmosChainBalances } from "./cosmos.ts"
import { getAptosChainBalances } from "./aptos.ts"
import { createQueries } from "@tanstack/svelte-query"

export type AssetMetadata = {
  denom: string
  display_symbol: string | null
  display_name: string | null
  decimals: number
  gasToken: boolean
  chain_id: string
  metadata_level: "graphql" | "onchain" | "none"
}

export type BalanceData = {
  balance: string
  metadata: AssetMetadata
}

function normalizeAddress(denom: string): string {
  return isAddress(denom) ? denom.toLowerCase() : denom
}

async function getAssetInfo(chain: Chain, denom: string): Promise<AssetMetadata> {
  try {
    const normalizedDenom = normalizeAddress(denom)
    const configAsset = chain.assets.find(
      (asset: ChainAsset) => normalizeAddress(asset.denom) === normalizedDenom
    )

    if (configAsset) {
      return {
        chain_id: chain.chain_id,
        denom: normalizedDenom,
        display_symbol: configAsset.display_symbol || null,
        display_name: configAsset.display_name || null,
        decimals: configAsset.decimals !== undefined ? configAsset.decimals : 0,
        gasToken: configAsset.gas_token,
        metadata_level: "graphql"
      }
    }

    if (chain.rpc_type === "evm" && isAddress(normalizedDenom)) {
      try {
        const results = await erc20ReadMulticall({
          chainId: chain.chain_id,
          functionNames: ["decimals", "symbol", "name"],
          address: normalizedDenom as Address,
          contractAddresses: [normalizedDenom] as Array<Address>
        })

        return {
          chain_id: chain.chain_id,
          denom: normalizedDenom,
          display_symbol: results[0].symbol ?? null,
          display_name: results[0].name ?? null,
          decimals: results[0].decimals ?? 18,
          gasToken: false,
          metadata_level: "onchain"
        }
      } catch (e) {
        console.error("Multicall metadata fetch failed:", e)
      }
    }

    return {
      chain_id: chain.chain_id,
      denom: normalizedDenom,
      display_symbol: null,
      display_name: null,
      decimals: 18,
      gasToken: false,
      metadata_level: "none"
    }
  } catch (error) {
    console.error("Unexpected error in getAssetInfo:", error)
    return {
      chain_id: chain.chain_id,
      denom: normalizeAddress(denom),
      display_symbol: null,
      display_name: null,
      decimals: 18,
      gasToken: false,
      metadata_level: "none"
    }
  }
}

function getAddressForChain(chain: Chain, addresses: UserAddresses): string | null {
  switch (chain.rpc_type) {
    case "evm":
      return addresses.evm?.canonical ?? null
    case "cosmos":
      return addresses.cosmos?.canonical ?? null
    case "aptos":
      return addresses.aptos?.canonical ?? null
    default:
      return null
  }
}

export function userBalancesQuery({
  userAddr,
  chains,
  connected = true
}: {
  userAddr: UserAddresses
  chains: Array<Chain>
  connected?: boolean
}) {
  return createQueries({
    queries: chains.map(chain => ({
      queryKey: [
        "balances",
        chain.chain_id,
        userAddr?.evm?.normalized,
        userAddr?.cosmos?.normalized,
        userAddr.aptos?.canonical
      ],
      refetchInterval: 4_000,
      refetchOnWindowFocus: false,
      queryFn: async () => {
        if (!connected) return []

        const address = getAddressForChain(chain, userAddr)
        if (!address) return []

        let rawBalances: Map<string, string> = new Map()

        if (chain.rpc_type === "evm") {
          const tokenList = chain.assets.filter(asset => isAddress(asset.denom))
          const multicallResults = await erc20ReadMulticall({
            chainId: chain.chain_id,
            functionNames: ["balanceOf"],
            address: address as Address,
            contractAddresses: tokenList.map(asset => asset.denom) as Array<Address>
          })

          multicallResults.forEach((result, index) => {
            rawBalances.set(tokenList[index].denom, result.balance?.toString() ?? "0")
          })
        } else if (chain.rpc_type === "cosmos") {
          const url = chain.rpcs.find(rpc => rpc.type === "rest")?.url
          if (!url) throw new Error(`No REST RPC available for chain ${chain.chain_id}`)

          const bech32Address = bech32ToBech32Address({
            toPrefix: chain.addr_prefix,
            address: address
          })

          const cosmosBalances = await getCosmosChainBalances({ url, walletAddress: bech32Address })
          cosmosBalances.forEach(balance => {
            rawBalances.set(balance.address, balance.balance.toString())
          })
        } else if (chain.rpc_type === "aptos") {
          const url = chain.rpcs.find(rpc => rpc.type === "rpc")?.url
          if (!url) throw new Error(`No RPC available for chain ${chain.chain_id}`)

          const aptosBalances = await getAptosChainBalances({ url, walletAddress: address })
          aptosBalances.forEach(balance => {
            rawBalances.set(balance.address, balance.balance.toString())
          })
        }

        // Convert all assets to BalanceData format, including those with zero balance
        const balances: Array<BalanceData> = await Promise.all(
          chain.assets.map(async asset => {
            const balance = rawBalances.get(asset.denom) ?? "0"
            return {
              balance,
              metadata: await getAssetInfo(chain, normalizeAddress(asset.denom))
            }
          })
        )

        // Sort balances: non-zero balances first, then by balance amount (descending)
        return balances.sort((a, b) => {
          const aValue = BigInt(a.balance)
          const bValue = BigInt(b.balance)
          if (aValue === 0n && bValue === 0n) return 0
          if (aValue === 0n) return 1
          if (bValue === 0n) return -1
          return bValue > aValue ? 1 : -1
        })
      }
    }))
  })
}
