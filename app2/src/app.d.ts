import type { Window as KeplrWindow } from "@keplr-wallet/types"
import type { LeapWindow } from "@leapwallet/types"

type FeaturesQuery = ReturnType<typeof enabledFeatures>
type Features = NonNullable<FeaturesQuery["data"]>["v1_ibc_union_chains"]

declare global {
  namespace App {
    // interface Locals {
    //   session: Option.Option<Session>
    // }
    // interface PageData {
    //   session: Option.Option<Session>
    // }
  }

  interface Window extends KeplrWindow, LeapWindow, Browser {
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
