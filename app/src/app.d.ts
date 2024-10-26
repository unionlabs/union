import "@tanstack/svelte-table"
import type { LeapWindow } from "@leapwallet/types"
import type { AptosBrowserWallet } from "@unionlabs/client"
import type { Window as KeplrWindow } from "@keplr-wallet/types"

declare module "@tanstack/svelte-table" {
  interface ColumnMeta<TData extends RowData, TValue> {
    class: string
  }
}

interface AptosWindow {
  aptos: AptosBrowserWallet
  petra: AptosBrowserWallet
}

declare global {
  namespace App {}

  interface Window extends AptosWindow, KeplrWindow, LeapWindow, Browser, GoogleRecaptcha {
    EventEmitter: typeof EventEmitter
  }

  interface Navigator {
    brave: {
      isBrave: () => Promise<boolean>
    }
  }
}

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
