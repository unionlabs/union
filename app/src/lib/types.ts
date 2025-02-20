import type { Address, Hex } from "viem"

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
  relayer_status: {
    status: string
    message: string
  }
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
  features: ChainFeature["features"]
  tokens: Array<ChainToken>
}

export type Ucs03Channel = {
  source_connection_id: number
  source_channel_id: number
  source_chain_id: string
  source_port_id: Hex
  destination_connection_id: number
  destination_channel_id: number
  destination_chain_id: string
  destination_port_id: Hex
}

export type ChainToken = {
  denom: string
  cw20: {
    cw20_token_address: string
  } | null
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
  sources: Array<unknown>
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
      decimals: number
      name: string
      symbol: string
    }
  | {
      quality_level: "NONE"
      denom: string
    }

export type TokenInfoMulti = {
  graphql: {
    primaryRepresentation: TokenRepresentation
    representations: Array<TokenRepresentation>
    wrapping: Array<Wrapping>
    cw20: {
      cw20_token_address: string
    } | null
  } | null
  onchain: {
    decimals: number | null
    name: string | null
    symbol: string | null
  } | null
  combined: {
    decimals: number
    symbol: string
    wrapping: Array<Wrapping>
  }
}

export interface ChainFeature {
  chain_id: string
  features: Array<{
    channel_list: boolean
    connection_list: boolean
    environment: string
    index_status: boolean
    packet_list: boolean
    transfer_list: boolean
    transfer_submission: boolean
  }>
}
