import { ofetch } from "ofetch"
import type { ChainId } from "../../mod.ts"

const queryHeaders = new Headers({
  Accept: "application/json",
  "User-Agent": "typescript-sdk",
  "Content-Type": "application/json"
})

const HUBBLE_URL = "https://graphql.union.build"

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
    includeContracts = false,
    includeAssets = false
  }: {
    includeEndpoints?: boolean
    includeContracts?: boolean
    includeAssets?: boolean
  } = {}) => {
    return await hubbleRestFetch<
      OffchainQueryBaseResponse<Chain<typeof includeEndpoints, typeof includeContracts>>
    >(`/chains`, {
      query: {
        include_rpcs: includeEndpoints,
        include_contracts: includeContracts,
        include_assets: includeAssets
      }
    })
  },
  chain: async ({
    chainId,
    includeEndpoints = false,
    includeContracts = false,
    includeAssets = false
  }: {
    chainId: string
    includeEndpoints?: boolean
    includeContracts?: boolean
    includeAssets?: boolean
  }) => {
    return await hubbleRestFetch<
      OffchainQueryBaseResponse<Chain<typeof includeEndpoints, typeof includeContracts>>
    >(`/chains/${chainId}`, {
      query: {
        include_assets: includeAssets,
        include_rpcs: includeEndpoints,
        include_contracts: includeContracts
      }
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

interface Asset {
  denom: string
  decimals: number
  gas_token: boolean
  display_symbol: string
  display_name: string | null
}

interface Forward {
  port: string
  channel_id: string
  connection_id: string
  contract_address: string
  destination_chain: {
    chain_id: string
  }
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
    chain_id: ChainId
    addr_prefix: string
    display_name: string
    logo_uri: string | null
  }
  forward: Array<Forward>
}

export interface Chain<
  IncludeEndpoints extends boolean | undefined = undefined,
  IncludeContracts extends boolean | undefined = undefined,
  IncludeAssets extends boolean | undefined = undefined
> {
  id: number
  testnet: boolean
  chain_id: string
  enabled: boolean
  addr_prefix: string
  display_name: string
  logo_uri: string | null
  rpc_type: "evm" | "cosmos"
  rpcs: IncludeEndpoints extends true ? Array<Rpc> : undefined
  assets: IncludeAssets extends true ? Array<Asset> : undefined
  ucs1_configurations: IncludeContracts extends true ? Array<Ucs1Configuration> : undefined
}
