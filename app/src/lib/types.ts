import type { Address } from "viem"

export interface TransferAsset {
  [symbol: string]: {
    amount: string
    info: {
      denom: string
      chain_id: string
      decimals: number
      logo_uri: string | null
      display_name: string | null
      display_symbol: string | null
    }
  }
}

export type UserAddresses = {
  cosmos: UserAddressCosmos | null
  evm: UserAddressEvm | null
  aptos: UserAddressAptos | null
}

export type UserAddressAptos = {
  canonical: string
  bytes: Uint8Array
}

export type UserAddressCosmos = {
  canonical: string
  normalized: string
  bytes: Uint8Array
  normalized_prefixed: Address
}

export type UserAddressEvm = {
  canonical: Address
  normalized: string
  normalized_prefixed: Address
}

export type Chain = {
  chain_id: string
  display_name: string
  testnet: boolean
  enabled: boolean
  enabled_staging: boolean
  rpc_type: "evm" | "cosmos" | "aptos"
  rpcs: Array<{
    type: string
    url: string
  }>
  explorers: Array<{
    tx_url: string
    block_url: string
    address_url: string
  }>
  addr_prefix: string
  assets: Array<ChainAsset>
  tokens: Array<ChainToken>
}

/** @deprecated */
export type ChainAsset = {
  denom: string
  display_symbol: string
  display_name: string | null
  decimals: number
  faucets: Array<{
    url: string
    display_name: string
  }>
  gas_token: boolean
}

export type ChainToken = {
  denom: string
  representations: Array<{
    name: string | null
    symbol: string | null
    decimals: number | null
    sources: Array<{
      id: string
      name: string
      logo_uri: string | null
      update_timestamp: string | null
    }>
  }>
  wrapping: Array<Wrapping>
}

export type Wrapping = {
  wrapped_chain: {
    chain_id: string
  }
  unwrapped_chain: {
    chain_id: string
  }
  destination_channel_id: number
  unwrapped_denom: string
}

export type TokenInfoQualityLevel = "GRAPHQL" | "ONCHAIN" | "NONE"

export type TokenRepresentation = {
  name: string
  symbol: string
  decimals: number
}
export type TokenInfo =
  | {
      quality_level: "GRAPHQL"
      denom: string
      primaryRepresentation: TokenRepresentation
      representations: Array<TokenRepresentation>
      wrapping: Array<Wrapping>
    }
  | {
      quality_level: "ONCHAIN"
      denom: string
      name: string
      symbol: string
    }
  | {
      quality_level: "NONE"
      denom: string
    }
