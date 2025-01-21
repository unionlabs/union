import { createQuery } from "@tanstack/svelte-query"
import { tokensQueryDocument } from "$lib/graphql/queries/tokens"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"
import type { Chain, TokenInfo, TokenInfoMulti } from "$lib/types"
import { getOnchainAssetInfo } from "./balance"
import { erc20ReadMulticall } from "./balance/evm/multicall"
import type { Address } from "viem"

export const tokensQuery = () =>
  createQuery({
    queryKey: ["tokens"],
    placeholderData: (previousData, _) => previousData,
    queryFn: async () =>
      (await request(URLS().GRAPHQL, tokensQueryDocument, {})).v1_ibc_union_tokens,
    enabled: true,
    refetchOnWindowFocus: false
  })

export const tokenInfoQuery = (chainId: string, denom: string, chains: Array<Chain>) =>
  createQuery({
    queryKey: ["token-info", chainId, denom],
    queryFn: async (): Promise<TokenInfoMulti> => {
      let tokenInfoMulti: TokenInfoMulti = {
        onchain: null,
        graphql: null,
        combined: { decimals: 0, symbol: denom, wrapping: [] }
      }
      let chain = chains.find(c => c.chain_id === chainId) ?? null
      if (chain === null) return tokenInfoMulti
      let graphqlToken = chain?.tokens.find(t => t.denom === denom) ?? null

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
          address: denom.toLowerCase() as Address,
          contractAddresses: [denom.toLowerCase()] as Array<Address>
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
    },
    staleTime: Number.POSITIVE_INFINITY,
    enabled: true,
    refetchOnWindowFocus: false
  })
