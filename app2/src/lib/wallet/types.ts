import type { State } from "@wagmi/core"
import type { Hex, Address as HexAddress } from "viem"
import {Schema} from "effect";
import type {RpcType} from "$lib/schema/chain.ts";

export type RpcTypeT = Schema.Schema.Type<typeof RpcType>

export type ChainWalletStore<TChainSource extends RpcTypeT> = {
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
