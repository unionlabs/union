import type { State } from "@wagmi/core"
import type { Hex, Address as HexAddress } from "viem"

/**
 * Represents a hexadecimal address with a "0x" prefix.
 */
export type EvmAddress = HexAddress

export type CosmosAddress = `union${string}`

/**
 * Represents an address that can be either a hexadecimal address or an Evmos address with a custom prefix.
 * @template TPrefix - The custom prefix for the Cosmos address.
 */
export type Address = EvmAddress | CosmosAddress

// shared types between wallets configs
export type ChainWalletStore<TChainSource extends "cosmos" | "evm" | "aptos"> = {
  chain:
    | (TChainSource extends "evm" ? "sepolia" : TChainSource extends "aptos" ? "aptos" : "cosmos")
    | String
  address: TChainSource extends "evm"
    ? EvmAddress | undefined
    : TChainSource extends "aptos"
      ? Hex | undefined
      : string | undefined
  rawAddress?: TChainSource extends "cosmos" ? Uint8Array | undefined : undefined
  connectionStatus: State["status"]
  connectedWallet: string | undefined
}

export type Msg = MsgSend

export interface MsgSend {
  fromAddress: CosmosAddress
  toAddress: CosmosAddress
  amount: Array<{ denom: string; amount: string }>
}
