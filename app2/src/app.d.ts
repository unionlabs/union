import type { Window as KeplrWindow } from "@keplr-wallet/types"
import type { LeapWindow } from "@leapwallet/types"
import type { AptosBrowserWallet } from "@unionlabs/client"

type FeaturesQuery = ReturnType<typeof enabledFeatures>
type Features = NonNullable<FeaturesQuery["data"]>["v1_ibc_union_chains"]

interface AptosWindow {
  aptos: AptosBrowserWallet
  petra: AptosBrowserWallet
  martian: AptosBrowserWallet
  okxwallet: {
    aptos: AptosBrowserWallet
  }
}

declare global {
  namespace App {
    // interface Locals {
    //   session: Option.Option<Session>
    // }
    // interface PageData {
    //   session: Option.Option<Session>
    // }
  }

  interface Window extends AptosWindow, KeplrWindow, LeapWindow, Browser {
    EventEmitter: typeof EventEmitter

    /** Guard to ensure GitHub logo proxy patches are applied only once */
    __githubLogoProxyPatched?: boolean
  }

  interface Navigator {
    brave: {
      isBrave: () => Promise<boolean>
    }
  }

  interface BigInt {
    toJSON(): string
  }
}

interface Browser {
  opr: unknown
  opera: { app: unknown }
  chrome: { app: unknown }
  safari: { pushNotification: unknown }
}
