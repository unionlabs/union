import type {LeapWindow} from "@leapwallet/types"
import type {AptosBrowserWallet} from "@unionlabs/client"
import type {Window as KeplrWindow} from "@keplr-wallet/types"

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
    //If we use +page/layout.ts for data
    // interface PageData {
    //
    // }
  }

  interface Window extends AptosWindow, KeplrWindow, LeapWindow, Browser {
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
