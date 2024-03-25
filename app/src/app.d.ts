import type { LeapWindow } from "@leapwallet/types"
import type { Window as KeplrWindow } from "@keplr-wallet/types"

declare global {
  namespace App {}
  interface Window extends KeplrWindow, LeapWindow {
    EventEmitter: typeof EventEmitter
  }
}

type EthereumRequestMethod =
  | "wallet_getSnaps"
  | "wallet_requestSnaps"
  | "wallet_invokeSnap"
  | "wallet_watchAsset"
