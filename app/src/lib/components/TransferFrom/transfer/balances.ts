import type { Chain, TokenInfoMulti, UserAddresses } from "$lib/types.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { derived, type Readable, writable } from "svelte/store"
import { type Address, hexToString, isHex } from "viem"
import { erc20ReadMulticall } from "$lib/queries/balance/evm/multicall.ts"
import type { QueryObserverResult } from "@tanstack/query-core"

interface BalanceQueryResult {
  chain_id: string
  balances: Record<string, string>
}

export const balanceStore = writable<Array<QueryObserverResult<BalanceQueryResult | null, Error>>>(
  []
)

export let userAddress: Readable<UserAddresses> = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$userAddrCosmos, $userAddrEvm, $userAddressAptos]) => ({
    evm: $userAddrEvm,
    cosmos: $userAddrCosmos,
    aptos: $userAddressAptos
  })
)

export async function getTokenInfoSimple(
  chainId: string,
  denom: string,
  chains: Array<Chain>
): Promise<TokenInfoMulti> {
  let denomDecoded = denom

  let tokenInfoMulti: TokenInfoMulti = {
    onchain: null,
    graphql: null,
    combined: { decimals: 0, symbol: denomDecoded, wrapping: [] }
  }

  let chain = chains.find(c => c.chain_id === chainId) ?? null
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
