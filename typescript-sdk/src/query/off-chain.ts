import { ofetch } from "ofetch"

/**
 * TODO:
 * - Figure out why graphql.union.build/api/rest/* gets a cloudflare challenge
 */

const HUBBLE_URL = import.meta.env.HUBBLE_URL ?? "https://noble-pika-27.hasura.app"

const queryHeaders = new Headers({
  Accept: "application/json",
  "Content-Type": "application/json",
  "User-Agent": "typescript-sdk"
})

const hubbleRestFetch = ofetch.create({
  method: "GET",
  retry: 2,
  retryDelay: 500,
  timeout: 6_000,
  headers: queryHeaders,
  baseURL: `${HUBBLE_URL}/api/rest`
})

export const offchainQuery = {
  chains: async ({
    includeEndpoints = false,
    includeContracts = false
  }: { includeEndpoints?: boolean; includeContracts?: boolean } = {}) => {
    return await hubbleRestFetch<
      OffchainQueryBaseResponse<Chain<typeof includeEndpoints, typeof includeContracts>>
    >(`/chains`, {
      query: { include_rpcs: includeEndpoints, include_contracts: includeContracts }
    })
  },
  chain: async ({
    chainId,
    includeEndpoints = false,
    includeContracts = false
  }: {
    chainId: string
    includeEndpoints?: boolean
    includeContracts?: boolean
  }) => {
    return await hubbleRestFetch<
      OffchainQueryBaseResponse<Chain<typeof includeEndpoints, typeof includeContracts>>
    >(`/chains/${chainId}`, {
      query: { include_rpcs: includeEndpoints, include_contracts: includeContracts }
    })
  }
}

interface OffchainQueryBaseResponse<T> {
  data: Array<T>
}

export interface Rpc {
  url: string
  enabled: boolean
  type: "rpc" | "rest" | "grpc"
}

export interface Ucs1Configuration {
  channel_id: string
  connection_id: string
  source_chain_id: number
  contract_address: string
  source_chain: {
    id: number
    testnet: boolean
    enabled: boolean
    chain_id: string
    addr_prefix: string
    display_name: string
    logo_uri: string | null
  }
  destination_chain_id: number
  destination_chain: {
    id: number
    testnet: boolean
    enabled: boolean
    chain_id: string
    addr_prefix: string
    display_name: string
    logo_uri: string | null
  }
}

export interface Chain<IncludeEndpoints extends boolean, IncludeContracts extends boolean> {
  id: number
  chain_id: string
  enabled: boolean
  addr_prefix: string
  display_name: string
  logo_uri: string | null
  rpc_type: "evm" | "cosmos"
  rpcs: IncludeEndpoints extends true ? Array<Rpc> : undefined
  ucs1_configurations: IncludeContracts extends true ? Array<Ucs1Configuration> : undefined
}
