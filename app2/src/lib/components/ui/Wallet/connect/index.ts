import type { State } from "@wagmi/core"
import type { EvmWalletId } from "../../../../wallet/evm.ts"
import type { AptosWalletId } from "../../../../wallet/aptos.ts"
import type { CosmosWalletId } from "../../../../wallet/cosmos.ts"

type Chain = "evm" | "cosmos" | "aptos"
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
  connectedWalletId:
    | (TChain extends "cosmos"
        ? CosmosWalletId
        : TChain extends "aptos"
          ? AptosWalletId
          : EvmWalletId)
    | null
    | undefined
  onConnectClick: (walletIdentifier: string) => void | Promise<void>
  onDisconnectClick: () => void
}

export type { Props, Props as ConnectProps }
