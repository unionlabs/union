import type { CosmosWalletId } from "$lib/wallet/cosmos"
import type { EvmWalletId } from "$lib/wallet/evm"
import type { SuiWalletId } from "$lib/wallet/sui"
import type { RpcType } from "@unionlabs/sdk/schema"
import type { State } from "@wagmi/core"
import type { Schema } from "effect"

type Chain = Schema.Schema.Type<typeof RpcType>
type ChainConnectStatus = State["status"]
type ChainWalletsInformation = ReadonlyArray<{
  id: string
  name: string
  icon: string
  download: string
}>

type Props<TChain extends Chain = Chain> = {
  chain: TChain
  address: string | undefined
  connectStatus: ChainConnectStatus
  chainWalletsInformation: ChainWalletsInformation
  connectedWalletId?:
    | (TChain extends "cosmos" ? CosmosWalletId
      : TChain extends "evm" ? EvmWalletId
      : TChain extends "sui" ? SuiWalletId
      : never)
    | undefined
  onConnectClick: (walletIdentifier: string) => void | Promise<void>
  onDisconnectClick: () => void
}

export type { Props, Props as ConnectProps }
