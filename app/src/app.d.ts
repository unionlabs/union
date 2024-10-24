import "@tanstack/svelte-table"
import type { LeapWindow } from "@leapwallet/types"
import type { Window as KeplrWindow } from "@keplr-wallet/types"

declare module "@tanstack/svelte-table" {
  interface ColumnMeta<TData extends RowData, TValue> {
    class: string
  }
}

interface Aptos {
  disconnect: () => Promise<void>
  isConnected: () => Promise<boolean>
  network: () => Promise<"Testnet" | "Mainnet">
  connect: () => Promise<{ address: string; publicKey: string }>
  account: () => Promise<{ address: string; publicKey: string }>
  getAccount: () => Promise<{ address: string; publicKey: string }>
  getNetwork: () => Promise<{ chainId: string; name: "Testnet" | "Mainnet"; url: string }>

  onAccountChange: (
    callback: (account: { address: string; publicKey: string; type?: unknown }) => void
  ) => void

  onNetworkChange: (
    callback: (network: { chainId: string; name: "Testnet" | "Mainnet"; url: string }) => void
  ) => void
}

declare global {
  namespace App {}

  interface Window extends KeplrWindow, LeapWindow, Browser, GoogleRecaptcha {
    aptos: Aptos
    petra: Aptos
    EventEmitter: typeof EventEmitter
  }

  interface Navigator {
    brave: {
      isBrave: () => Promise<boolean>
    }
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

interface Browser {
  opr: unknown
  opera: { app: unknown }
  chrome: { app: unknown }
  safari: { pushNotification: unknown }
}

interface GoogleRecaptcha {
  __google_recaptcha_client: boolean
  grecaptcha: {
    execute: (siteKey: string, options: { action: string }) => Promise<string>
    enterprise: {
      execute: (siteKey: string, options: { action: string }) => Promise<string>
    }
  }
}
