import { type Address, isAddress } from "viem"
import { bech32ToBech32Address } from "@unionlabs/client"
import type { Chain, ChainAsset, TokenInfo, UserAddresses } from "$lib/types"
import { erc20ReadMulticall } from "./evm/multicall.ts"
import { getCosmosChainBalances } from "./cosmos.ts"
import { getAptosChainBalances } from "./aptos.ts"
import { createQueries } from "@tanstack/svelte-query"
import { err, errAsync, ok, ResultAsync, type Result } from "neverthrow"

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
  denom: string
  balance: string
}

function normalizeAddress(denom: string): string {
  return isAddress(denom) ? denom.toLowerCase() : denom
}

export async function getOnchainAssetInfo(chain: Chain, denom: string): Promise<TokenInfo> {
  const normalizedDenom = normalizeAddress(denom)
  if (chain.rpc_type === "evm") {
    try {
      const results = await erc20ReadMulticall({
        chainId: chain.chain_id,
        functionNames: ["decimals", "symbol", "name"],
        address: normalizedDenom as Address,
        contractAddresses: [normalizedDenom] as Array<Address>
      })
      return {
        quality_level: "ONCHAIN",
        denom,
        name: results[0].name,
        decimals: results[0].decimals,
        symbol: results[0].symbol
      }
    } catch (e) {
      return {
        quality_level: "NONE",
        denom
      }
    }
  }
  return {
    quality_level: "NONE",
    denom
  }
}

export async function getAssetInfo(chain: Chain, denom: string): Promise<AssetMetadata> {
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
    } else if (chain.rpc_type === "cosmos") {
      // TODO: fetch proper balance metadata from chain.
      return {
        chain_id: chain.chain_id,
        denom: normalizedDenom,
        display_symbol: denom,
        display_name: denom,
        decimals: 0,
        gasToken: false,
        metadata_level: "onchain"
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

export type Denom = string
export type RawBalances = Record<Denom, string>

export function userBalancesQuery({
  userAddr,
  chains
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
      keepPreviousData: true,
      enabled: Boolean(getAddressForChain(chain, userAddr)),
      refetchInterval: 4_000,
      refetchOnWindowFocus: false,
      queryFn: async (): Promise<{ chain_id: string; balances: Result<RawBalances, Error> }> => {
        const address = getAddressForChain(chain, userAddr)
        if (!address) {
          return {
            chain_id: chain.chain_id,
            balances: err(new Error(`no user address for chain ${chain.chain_id}`))
          }
        }

        if (chain.rpc_type === "evm") {
          const tokenList = chain.tokens.filter(tokens => isAddress(tokens.denom))
          return {
            chain_id: chain.chain_id,
            balances: await ResultAsync.fromPromise(
              erc20ReadMulticall({
                chainId: chain.chain_id,
                functionNames: ["balanceOf"],
                address: address as Address,
                contractAddresses: tokenList.map(asset => asset.denom) as Array<Address>
              }),
              error => new Error("error fetching evm balances", { cause: error })
            ).andThen(multicallResultss =>
              ok(
                multicallResultss.reduce((acc, curr, index) => {
                  if (curr.balance) {
                    acc[tokenList[index].denom] = curr.balance.toString()
                  }
                  return acc
                }, {})
              )
            )
          }
        }

        if (chain.rpc_type === "cosmos") {
          const url = chain.rpcs.find(rpc => rpc.type === "rest")?.url
          if (!url) {
            return {
              chain_id: chain.chain_id,
              balances: err(new Error(`no rest url for cosmos chain ${chain.chain_id}`))
            }
          }

          const bech32Address = bech32ToBech32Address({
            toPrefix: chain.addr_prefix,
            address: address
          })

          return {
            chain_id: chain.chain_id,
            balances: await getCosmosChainBalances({ url, walletAddress: bech32Address })
          }
          // cosmosBalances.forEach(balance => {
          //   rawBalances[balance.address] = balance.balance.toString()
          // })
        }
        // if (chain.rpc_type === "aptos") {
        //   const url = chain.rpcs.find(rpc => rpc.type === "rpc")?.url
        //   if (!url) throw new Error(`No RPC available for chain ${chain.chain_id}`)

        //   const aptosBalances = await getAptosChainBalances({ url, walletAddress: address })
        //   aptosBalances.forEach(balance => {
        //     rawBalances[balance.address] = balance.balance.toString()
        //   })
        // }

        // return { chain_id: chain.chain_id, balances: rawBalances }
        return { chain_id: chain.chain_id, balances: err(new Error("unimplemented")) }
      }
    }))
  })
}
