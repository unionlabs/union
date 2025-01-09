import { writable, derived, type Readable } from "svelte/store"
import { bech32ToBech32Address } from "@unionlabs/client"
import { type Address, isAddress } from "viem"
import type { Chain, ChainAsset, UserAddresses } from "$lib/types"
import { erc20ReadMulticall } from "./evm/multicall.ts"
import { getCosmosChainBalances } from "./cosmos.ts"
import { getAptosChainBalances } from "./aptos.ts"
import { createQueries } from "@tanstack/svelte-query"
import type { QueryObserverResult } from "@tanstack/query-core"

export type AssetMetadata = {
  balance: string
  denom: string
  display_symbol: string | null
  display_name: string | null
  decimals: number | null
  gasToken: boolean
  metadata_level: "graphql" | "onchain" | "none"
}

export type BalanceData = {
  denom: string
  balance: string
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
        balance: "0",
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
          balance: "0",
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
      balance: "0",
      denom: normalizedDenom,
      display_symbol: null,
      display_name: null,
      decimals: null,
      gasToken: false,
      metadata_level: "none"
    }
  } catch (error) {
    console.error("Unexpected error in getAssetInfo:", error)
    return {
      balance: "0",
      denom: normalizeAddress(denom),
      display_symbol: null,
      display_name: null,
      decimals: null,
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

      return results
        .map((result, index) => ({
          denom: normalizeAddress(contractAddresses[index]),
          balance: result.balance?.toString() ?? "0"
        }))
        .filter(result => BigInt(result.balance) > 0n)
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

      return balances.map(balance => ({
        denom: normalizeAddress(balance.address),
        balance: balance.balance.toString()
      }))
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

      return balances.map(balance => ({
        denom: normalizeAddress(balance.address),
        balance: balance.balance.toString()
      }))
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
): Readable<Array<AssetMetadata>> {
  const balanceStore = writable<Array<AssetMetadata>>([])

  return derived<Readable<UserAddresses>, Array<AssetMetadata>>(
    addressStore,
    ($addresses, set) => {
      const address = getAddressForChain(chain, $addresses)

      if (!address) {
        set([])
        return
      }

      const initialBalances: Array<AssetMetadata> = chain.assets.map(asset => ({
        balance: "0",
        denom: asset.denom,
        display_symbol: asset.display_symbol || null,
        display_name: asset.display_name || null,
        decimals: asset.decimals !== undefined ? asset.decimals : null,
        gasToken: asset.gas_token,
        metadata_level: "none"
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
                const enrichedBalances = await Promise.all(
                  balances.map(async ({ denom, balance }) => {
                    const assetInfo = await getAssetInfo(chain, denom)
                    return { ...assetInfo, balance, denom }
                  })
                )

                // Merge with placeholder balances to ensure all assets are represented
                const mergedBalances = initialBalances.map(placeholder => {
                  const enriched = enrichedBalances.find(b => b.denom === placeholder.denom)
                  return enriched || placeholder
                })

                // Add any new tokens discovered that weren't in the original asset list
                enrichedBalances.forEach(enriched => {
                  if (!mergedBalances.some(b => b.denom === enriched.denom)) {
                    mergedBalances.push(enriched)
                  }
                })

                // Ensure all balance values are valid numbers and sort the balances
                return mergedBalances
                  .map(balance => ({
                    ...balance,
                    balance: balance.balance === "Loading..." ? "0" : balance.balance,
                    decimals: balance.decimals !== null ? balance.decimals : 18 // Default to 18 if decimals is null
                  }))
                  .sort((a, b) => {
                    const aValue = BigInt(a.balance) * BigInt(10 ** (18 - a.decimals))
                    const bValue = BigInt(b.balance) * BigInt(10 ** (18 - b.decimals))
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
        const queryResult = results[0] as QueryObserverResult<Array<AssetMetadata>, Error>
        if (queryResult.data) {
          balanceStore.set(queryResult.data)
          set(queryResult.data)
        }
      })

      return balanceStore.subscribe(set)
    },
    [] as Array<AssetMetadata>
  )
}

export function allChainBalances(chains: Array<Chain>, addressStore: Readable<UserAddresses>) {
  const chainStores = chains.map(chain => createChainBalances(chain, addressStore))

  return derived(chainStores, $chainStores => $chainStores)
}
