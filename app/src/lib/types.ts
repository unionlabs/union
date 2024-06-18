import type { Address } from "viem"

export type UserAddresses = {
  cosmos: {
    canonical: string
    normalized: string
    bytes: Uint8Array
    normalized_prefixed: Address
  }
  evm: {
    canonical: Address
    normalized: string
    normalized_prefixed: Address
  }
}

export type Chain = {
  chain_id: string
  display_name: string
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
  addr_prefix: string
  assets: Array<{
    denom: string
    display_symbol: string
    decimals: number
    faucets: Array<{
      url: string
      display_name: string
    }>
  }>
}
