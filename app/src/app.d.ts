import "@tanstack/svelte-table"
import type { LeapWindow } from "@leapwallet/types"
import type { Window as KeplrWindow } from "@keplr-wallet/types"

declare module "@tanstack/svelte-table" {
  interface ColumnMeta<TData extends RowData, TValue> {
    class: string
  }
}

declare global {
  namespace App {}

  namespace Superforms {
    type Message = {
      text: string
      type: "error" | "success"
    }
  }

  interface Window extends KeplrWindow, LeapWindow {
    EventEmitter: typeof EventEmitter
    __google_recaptcha_client: boolean
    grecaptcha: {
      execute: (siteKey: string, options: { action: string }) => Promise<string>
      enterprise: {
        execute: (siteKey: string, options: { action: string }) => Promise<string>
      }
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
