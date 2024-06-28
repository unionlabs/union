import type { Address } from "viem"

export type DiscriminatedUnion<K extends PropertyKey, T extends object> = {
  [P in keyof T]: { [Q in K]: P } & T[P] extends infer U ? { [Q in keyof U]: U[Q] } : never
}[keyof T]


export type UserAddresses = {
  cosmos: UserAddressCosmos | null
  evm: UserAddressEvm | null
  [key: string]: UserAddressCosmos | UserAddressEvm | null
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
  rpc_type: "evm" | "cosmos"
  rpcs: Array<{
    type: string
    url: string
  }>
  ucs1_configurations: Record<
    string,
    {
      contract_address: string
      channel_id: string
      forward: Record<
        string,
        {
          channel_id: string
          port: string
        }
      >
    }
  >
  explorers: Array<{
    tx_url: string
    block_url: string
    address_url: string
  }>
  addr_prefix: string
  assets: Array<{
    denom: string
    display_symbol: string
    display_name: string | null
    decimals: number
    faucets: Array<{
      url: string
      display_name: string
    }>
  }>
}
