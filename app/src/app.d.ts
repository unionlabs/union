import type { LeapWindow } from "@leapwallet/types"
import type { Window as KeplrWindow } from "@keplr-wallet/types"

declare global {
  namespace App {}
  interface Window extends KeplrWindow, LeapWindow {
    EventEmitter: typeof EventEmitter
  }
  interface WindowEventMap {
    "eip6963:announceProvider": CustomEvent
  }
  interface EIP6963ProviderInfo {
    rdns: string
    uuid: string
    name: string
    icon: string
  }

  interface EIP6963ProviderDetail {
    info: EIP6963ProviderInfo
    provider: EIP1193Provider
  }

  type EIP6963AnnounceProviderEvent = {
    detail: {
      info: EIP6963ProviderInfo
      provider: Readonly<EIP1193Provider>
    }
  }

  interface EIP1193Provider {
    isStatus?: boolean
    host?: string
    path?: string
    sendAsync?: (
      request: { method: string; params?: Array<unknown> },
      callback: (error: Error | null, response: unknown) => void
    ) => void
    send?: (
      request: { method: string; params?: Array<unknown> },
      callback: (error: Error | null, response: unknown) => void
    ) => void
    request: (request: { method: string; params?: Array<unknown> }) => Promise<unknown>
  }
}

type EthereumRequestMethod =
  | "wallet_getSnaps"
  | "wallet_requestSnaps"
  | "wallet_invokeSnap"
  | "wallet_watchAsset"
