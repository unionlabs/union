import type { Chain, TokenInfoMulti } from "$lib/types.ts"
import { get, writable, type Writable } from "svelte/store"
import type { ChainId, Denom } from "./balances.ts" // hack, move to proper place
import { hexToString, isHex, type Address } from "viem"
import { erc20ReadMulticall } from "$lib/queries/balance/evm/multicall"

type TokenInfos = Record<ChainId, Record<Denom, TokenInfo>>

export type TokenInfo =
  | { kind: "loading" }
  | { kind: "tokenInfo"; info: TokenInfoMulti | null; timestamp: number }
  | { kind: "error"; error: string; timestamp: number }

export let tokenInfos: Writable<TokenInfos> = writable({})

export async function requestTokenInfo(chain: Chain, denom: Denom) {
  if (
    get(tokenInfos)[chain.chain_id] &&
    (get(tokenInfos)[chain.chain_id][denom]?.kind === "loading" ||
      get(tokenInfos)[chain.chain_id][denom]?.kind === "tokenInfo")
  ) {
    // we already have this info
    console.log("token info cache hit", chain.chain_id, denom)
    return
  }

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

    if (fullRepresentations.length > 0) {
      tokenInfoMulti.graphql = {
        primaryRepresentation: fullRepresentations[0],
        representations: fullRepresentations,
        wrapping: graphqlToken.wrapping
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

  let graphqlRepr = tokenInfoMulti.graphql?.primaryRepresentation
  if (graphqlRepr) {
    tokenInfoMulti.combined.symbol = graphqlRepr.symbol
    tokenInfoMulti.combined.decimals = graphqlRepr.decimals
  } else if (tokenInfoMulti.onchain?.symbol && tokenInfoMulti.onchain.decimals) {
    tokenInfoMulti.combined.symbol = tokenInfoMulti.onchain.symbol
    tokenInfoMulti.combined.decimals = tokenInfoMulti.onchain.decimals
  }

  return tokenInfoMulti
}
