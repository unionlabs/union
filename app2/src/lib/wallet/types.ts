import type { State } from "@wagmi/core"
import type { Hex, Address as HexAddress } from "viem"

export type ChainWalletStore<TChainSource extends "cosmos" | "evm" | "aptos"> = {
  chain:
    | (TChainSource extends "evm" ? "sepolia" : TChainSource extends "aptos" ? "aptos" : "cosmos")
    | string
  address: TChainSource extends "evm"
    ? HexAddress | undefined
    : TChainSource extends "aptos"
      ? Hex | undefined
      : string | undefined
  rawAddress?: TChainSource extends "cosmos" ? Uint8Array | undefined : undefined
  connectionStatus: State["status"]
  connectedWallet: string | undefined
}
