import type { Chain, TokenInfoMulti } from "$lib/types.ts"
import { get, type Writable } from "svelte/store"
import type { ChainId, Denom } from "./balances.ts" // hack, move to proper place
import { fromHex, hexToString, isHex, type Address } from "viem"
import { erc20ReadMulticall } from "$lib/queries/balance/evm/multicall"
import {
  isValidBech32ContractAddress,
  queryCosmosC20TokenMetadata,
  type CosmosChainId
} from "@unionlabs/client"
import { persisted } from "svelte-persisted-store"

export type TokenInfos = Record<ChainId, Record<Denom, TokenInfo>>

export type TokenInfo =
  | { kind: "loading" }
  | { kind: "tokenInfo"; info: TokenInfoMulti | null; timestamp: number }
  | { kind: "error"; error: string; timestamp: number }

export let tokenInfos: Writable<TokenInfos> = persisted("token-infos", {})

function isStale(info: TokenInfo): boolean {
  if (info.kind !== "tokenInfo") return false
  const oneMinute = 60 * 60 * 1000 // 1 hour in milliseconds
  return Date.now() - info.timestamp > oneMinute
}

export async function requestTokenInfo(chain: Chain, denom: Denom) {
  if (
    get(tokenInfos)[chain.chain_id] &&
    (get(tokenInfos)[chain.chain_id][denom]?.kind === "loading" ||
      get(tokenInfos)[chain.chain_id][denom]?.kind === "tokenInfo")
  ) {
    const tokenInfo = get(tokenInfos)[chain.chain_id][denom]
    if (isStale(tokenInfo)) {
      console.info("[TokenInfo] stale info", chain.chain_id, denom)
      tokenInfos.update(val => {
        delete val[chain.chain_id][denom]
        return val
      })
    } else {
      console.info("[TokenInfo] cache hit", chain.chain_id, denom)
      return
    }
  }

  console.info("[TokenInfo] fetching new info", chain.chain_id, denom)

  tokenInfos.update(val => {
    if (val[chain.chain_id] === undefined) {
      val[chain.chain_id] = {}
    }
    val[chain.chain_id][denom] = { kind: "loading" }
    return val
  })

  let tokenInfo = await fetchTokenInfo(chain, denom)

  tokenInfos.update(val => {
    val[chain.chain_id][denom] = { kind: "tokenInfo", info: tokenInfo, timestamp: Date.now() }
    return val
  })
}

export async function fetchTokenInfo(chain: Chain, denom: Denom): Promise<TokenInfoMulti> {
  let denomDecoded = denom

  let tokenInfoMulti: TokenInfoMulti = {
    onchain: null,
    graphql: null,
    combined: { decimals: 0, symbol: denomDecoded, wrapping: [] }
  }
  if (chain === null) return tokenInfoMulti
  if (chain.rpc_type === "cosmos" && isHex(denom)) {
    denomDecoded = hexToString(denom)
    tokenInfoMulti.combined.symbol = denomDecoded
  }

  // note the non-decoded denom is used
  let graphqlToken = chain?.tokens.find(t => t.denom === denom) ?? null

  if (graphqlToken?.wrapping && graphqlToken.wrapping.length > 0) {
    tokenInfoMulti.combined.wrapping = graphqlToken.wrapping
  }

  // GraphQL info
  if (graphqlToken?.representations && graphqlToken.representations.length > 0) {
    let fullRepresentations = graphqlToken.representations.filter(
      repr => repr.decimals != null && repr.name != null && repr.symbol != null
    ) as Array<
      {
        decimals: number
        name: string
        symbol: string
      } & (typeof graphqlToken.representations)[number]
    >

    if (graphqlToken.cw20) {
      console.log("cw20 found", graphqlToken.cw20)
    }
    if (fullRepresentations.length > 0) {
      tokenInfoMulti.graphql = {
        primaryRepresentation: fullRepresentations[0],
        representations: fullRepresentations,
        wrapping: graphqlToken.wrapping,
        cw20: graphqlToken.cw20
      }
      tokenInfoMulti.combined.wrapping = graphqlToken.wrapping
    }
  }

  // Onchain info
  if (chain.rpc_type === "evm") {
    const results = await erc20ReadMulticall({
      chainId: chain.chain_id,
      functionNames: ["decimals", "symbol", "name"],
      address: denomDecoded.toLowerCase() as Address,
      contractAddresses: [denomDecoded.toLowerCase()] as Array<Address>
    })

    tokenInfoMulti.onchain = {
      name: results[0].name,
      decimals: results[0].decimals,
      symbol: results[0].symbol
    }
  }

  if (chain.rpc_type === "cosmos") {
    const maybeBechAddr = isHex(denom) ? fromHex(denom, "string") : ""
    if (isValidBech32ContractAddress(maybeBechAddr)) {
      const result = await queryCosmosC20TokenMetadata({
        contractAddress: maybeBechAddr,
        chainId: chain.chain_id as CosmosChainId
      })

      if (result.isOk()) {
        tokenInfoMulti.onchain = {
          name: result.value.name,
          decimals: result.value.decimals,
          symbol: result.value.symbol
        }
      }

      if (result.isErr()) {
        console.error(`error getting metadata for asset, ${denom}`)
      }
    }
  }

  let graphqlRepr = tokenInfoMulti.graphql?.primaryRepresentation
  if (graphqlRepr) {
    tokenInfoMulti.combined.symbol = graphqlRepr.symbol
    tokenInfoMulti.combined.decimals = graphqlRepr.decimals
  } else {
    if (tokenInfoMulti.onchain?.symbol) {
      tokenInfoMulti.combined.symbol = tokenInfoMulti.onchain.symbol
    }
    if (tokenInfoMulti.onchain?.decimals) {
      tokenInfoMulti.combined.decimals = tokenInfoMulti.onchain.decimals
    }
  }

  return tokenInfoMulti
}
