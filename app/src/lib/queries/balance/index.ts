import { derived, type Readable, writable } from "svelte/store"
import { bech32ToBech32Address } from "@unionlabs/client"
import { type Address, isAddress } from "viem"
import type { Chain, ChainAsset, UserAddresses } from "$lib/types"
import { erc20ReadMulticall } from "./evm/multicall.ts"
import { getCosmosChainBalances } from "./cosmos.ts"
import { getAptosChainBalances } from "./aptos.ts"
import { createQueries } from "@tanstack/svelte-query"
import type { QueryObserverResult } from "@tanstack/query-core"

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

export async function getAssetInfo(chain: Chain, denom: string): Promise<AssetMetadata> {
  try {
    const normalizedDenom = normalizeAddress(denom)
    const configAsset = chain.assets.find(
      (asset: { denom: string }) => normalizeAddress(asset.denom) === normalizedDenom
    )

    if (configAsset) {
      return {
        chain_id: chain.chain_id, // Add this line
        denom: normalizedDenom,
        display_symbol: configAsset.display_symbol,
        display_name: configAsset.display_name,
        decimals: configAsset.decimals,
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
          chain_id: chain.chain_id, // Add this line
          denom: normalizedDenom,
          display_symbol: results[0].symbol ?? null,
          display_name: results[0].name ?? null,
          decimals: results[0].decimals ?? null,
          gasToken: false,
          metadata_level: "onchain"
        }
      } catch (e) {
        console.error("Multicall metadata fetch failed:", e)
      }
    }

    // Fallback
    return {
      chain_id: chain.chain_id, // Add this line
      denom: normalizedDenom,
      display_symbol: null,
      display_name: null,
      decimals: 0,
      gasToken: false,
      metadata_level: "none"
    }
  } catch (error) {
    console.error("Unexpected error in getAssetInfo:", error)
    return {
      chain_id: chain.chain_id, // Add this line
      denom: normalizeAddress(denom),
      display_symbol: null,
      display_name: null,
      decimals: 0,
      gasToken: false,
      metadata_level: "none"
    }
  }
}

export async function getUserBalances(
  chain: Chain,
  address: string,
  denoms?: Array<string>
): Promise<Array<BalanceData>> {
  try {
    if (chain.rpc_type === "evm") {
      const contractAddresses = denoms
        ? denoms.filter((denom): denom is Address => isAddress(denom)).map(normalizeAddress)
        : chain.assets
            .filter((asset): asset is ChainAsset & { denom: Address } => isAddress(asset.denom))
            .map(asset => normalizeAddress(asset.denom))

      const results = await erc20ReadMulticall({
        chainId: chain.chain_id,
        functionNames: ["balanceOf"],
        address: address as Address,
        contractAddresses: contractAddresses as Array<Address>
      })

      const balances = await Promise.all(
        results.map(async (result, index) => {
          const denom = normalizeAddress(contractAddresses[index])
          const balance = result.balance?.toString() ?? "0"
          const metadata = await getAssetInfo(chain, denom)
          return { balance, metadata }
        })
      )

      return balances.filter(result => BigInt(result.balance) > 0n)
    }

    if (chain.rpc_type === "cosmos") {
      const restEndpoint = chain.rpcs.find(rpc => rpc.type === "rest")?.url
      if (!restEndpoint) {
        console.error(`No REST endpoint found for chain ${chain.chain_id}`)
        return []
      }

      const bech32Address = bech32ToBech32Address({
        toPrefix: chain.addr_prefix,
        address
      })

      const balances = await getCosmosChainBalances({
        url: restEndpoint,
        walletAddress: bech32Address
      })

      return Promise.all(
        balances.map(async balance => ({
          balance: balance.balance.toString(),
          metadata: await getAssetInfo(chain, normalizeAddress(balance.address))
        }))
      )
    }

    if (chain.rpc_type === "aptos") {
      const graphqlEndpoint = chain.rpcs.find(rpc => rpc.type === "rpc")?.url
      if (!graphqlEndpoint) {
        console.error(`No GraphQL endpoint found for chain ${chain.chain_id}`)
        return []
      }

      const balances = await getAptosChainBalances({
        url: graphqlEndpoint,
        walletAddress: address
      })

      return Promise.all(
        balances.map(async balance => ({
          balance: balance.balance.toString(),
          metadata: await getAssetInfo(chain, normalizeAddress(balance.address))
        }))
      )
    }

    return []
  } catch (error) {
    console.error("Error in getUserBalances:", error)
    return []
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

export function createChainBalances(
  chain: Chain,
  addressStore: Readable<UserAddresses>
): Readable<Array<BalanceData>> {
  const balanceStore = writable<Array<BalanceData>>([])

  return derived<Readable<UserAddresses>, Array<BalanceData>>(
    addressStore,
    ($addresses, set) => {
      const address = getAddressForChain(chain, $addresses)

      if (!address) {
        set([])
        return
      }

      const initialBalances: Array<BalanceData> = chain.assets.map(asset => ({
        balance: "0",
        metadata: {
          denom: asset.denom,
          display_symbol: asset.display_symbol || null,
          display_name: asset.display_name || null,
          decimals: asset.decimals !== undefined ? asset.decimals : 0,
          gasToken: asset.gas_token,
          chain_id: chain.chain_id,
          metadata_level: "none"
        }
      }))
      balanceStore.set(initialBalances)
      set(initialBalances)

      // Fetch all balances
      createQueries({
        queries: [
          {
            queryKey: ["balances", chain.chain_id, address],
            queryFn: async () => {
              try {
                const balances = await getUserBalances(chain, address)

                // Merge with placeholder balances to ensure all assets are represented
                const mergedBalances = initialBalances.map(placeholder => {
                  const enriched = balances.find(
                    b => b.metadata.denom === placeholder.metadata.denom
                  )
                  return enriched || placeholder
                })

                // Add any new tokens discovered that weren't in the original asset list
                balances.forEach(balance => {
                  if (!mergedBalances.some(b => b.metadata.denom === balance.metadata.denom)) {
                    mergedBalances.push(balance)
                  }
                })

                // Ensure all balance values are valid numbers and sort the balances
                return mergedBalances
                  .map(balance => ({
                    ...balance,
                    balance: balance.balance === "Loading..." ? "0" : balance.balance,
                    metadata: {
                      ...balance.metadata,
                      decimals: balance.metadata.decimals !== null ? balance.metadata.decimals : 18 // Default to 18 if decimals is null
                    }
                  }))
                  .sort((a, b) => {
                    const aValue =
                      BigInt(a.balance) * BigInt(10 ** (18 - (a.metadata.decimals ?? 18)))
                    const bValue =
                      BigInt(b.balance) * BigInt(10 ** (18 - (b.metadata.decimals ?? 18)))
                    return bValue > aValue ? 1 : -1
                  })
              } catch (error) {
                console.error("Error fetching balances:", error)
                return initialBalances
              }
            },
            refetchInterval: 4000
          }
        ]
      }).subscribe(results => {
        const queryResult = results[0] as QueryObserverResult<Array<BalanceData>, Error>
        if (queryResult.data) {
          balanceStore.set(queryResult.data)
          set(queryResult.data)
        }
      })

      return balanceStore.subscribe(set)
    },
    [] as Array<BalanceData>
  )
}

export function allChainBalances(chains: Array<Chain>, addressStore: Readable<UserAddresses>) {
  const chainStores = chains.map(chain => createChainBalances(chain, addressStore))

  return derived(chainStores, $chainStores => $chainStores)
}
