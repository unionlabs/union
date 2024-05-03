import Connect from "./connect.svelte"
import type { State } from "@wagmi/core"
import type { EvmWalletName } from "$lib/wallet/evm"
import type { CosmosWalletName } from "$lib/wallet/cosmos"

type Chain = "evm" | "cosmos"
type HoverState = "hover" | "none"
type ChainConnectStatus = State["status"]
type ChainWalletsInformation = ReadonlyArray<{
  id?: string
  name: string
  icon: string
  download: string
}> // & any

type Props<TChain extends Chain = Chain> = {
  chain: TChain
  hoverState: HoverState
  address: string | undefined
  connectStatus: ChainConnectStatus
  chainWalletsInformation: ChainWalletsInformation
  connectedWalletId: (TChain extends "cosmos" ? CosmosWalletName : EvmWalletName) | null | undefined
  onConnectClick: (walletIdentifier: string) => void | Promise<void>
  onDisconnectClick: () => void
}

export { Connect, type Props, type Props as ConnectProps }
