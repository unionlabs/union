import { derived, get } from "svelte/store"
import { hexToBytes, type Hex } from "viem"
import { persisted } from "svelte-persisted-store"
import type { ChainWalletStore } from "../types.ts"
import type { AptosBrowserWallet } from "@unionlabs/client"

export const aptosWalletsInformation = (
  [
    // https://petra.app/docs
    {
      id: "petra",
      name: "Petra",
      icon: "/images/icons/petra.svg",
      download: "https://petra.app",
      deepLink: "https://petra.app/explore?link=https://app.union.build"
    },
    // https://docs.martianwallet.xyz/docs
    {
      id: "martian",
      name: "Martian Wallet",
      // could not find a good svg for martian
      icon: "/images/icons/martian.png",
      download: "https://martianwallet.xyz/aptos-wallet",
      deepLink: ""
    },
    // https://www.okx.com/web3/build/docs/sdks/chains/aptos/provider
    {
      id: "okxwallet",
      name: "OKX Wallet",
      icon: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAJDSURBVHgB7Zq9jtpAEMfHlhEgQLiioXEkoAGECwoKxMcTRHmC5E3IoyRPkPAEkI7unJYmTgEFTYwA8a3NTKScLnCHN6c9r1e3P2llWQy7M/s1Gv1twCP0ej37dDq9x+Zut1t3t9vZjDEHIiSRSPg4ZpDL5fxkMvn1cDh8m0wmfugfO53OoFQq/crn8wxfY9EymQyrVCqMfHvScZx1p9ls3pFxXBy/bKlUipGPrVbLuQqAfsCliq3zl0H84zwtjQrOw4Mt1W63P5LvBm2d+Xz+YzqdgkqUy+WgWCy+Mc/nc282m4FqLBYL+3g8fjDxenq72WxANZbLJeA13zDX67UDioL5ybXwafMYu64Ltn3bdDweQ5R97fd7GyhBQMipx4POeEDHIu2LfDdBIGGz+hJ9CQ1ABjoA2egAZPM6AgiCAEQhsi/C4jHyPA/6/f5NG3Ks2+3CYDC4aTccDrn6ojG54MnEvG00GoVmWLIRNZ7wTCwDHYBsdACy0QHIhiuRETxlICWpMMhGZHmqS8qH6JLyGegAZKMDkI0uKf8X4SWlaZo+Pp1bRrwlJU8ZKLIvUjKh0WiQ3sRUbNVq9c5Ebew7KEo2m/1p4jJ4qAmDaqDQBzj5XyiAT4VCQezJigAU+IDU+z8vJFnGWeC+bKQV/5VZ71FV6L7PA3gg3tXrdQ+DgLhC+75Wq3no69P3MC0NFQpx2lL04Ql9gHK1bRDjsSBIvScBnDTk1WrlGIZBorIDEYJj+rhdgnQ67VmWRe0zlplXl81vcyEt0rSoYDUAAAAASUVORK5CYII=",
      download: "https://www.okx.com/download",
      deepLink: `https://www.okx.com/download?deeplink=${encodeURIComponent(`okx://wallet/dapp/url?dappUrl=${encodeURIComponent(window.location.href)}`)}`
    }
  ] as const
)
  /**
   * Temporary filtering out reasons:
   *
   * `okxwallet` because it uses the deprecated payload API where
   * instead of `.signAndSubmitTransaction({ payload: ... })`, it uses `.signAndSubmitTransaction({ ... })`.
   * In our SDK we use the current, non-deprecated API.
   *
   * `martian` because it doesn't implement `.getNetwork()` and `.getAccount()` methods yet.
   * We will stop filtering them out once they use the new API.
   */
  .filter(wallet => !["okxwallet", "martian"].includes(wallet.id))

export type AptosWalletId = (typeof aptosWalletsInformation)[number]["id"]

/**
 * TODO:
 * - check with Petra wallet team for proper `window.aptos` types
 */

export function getAptosWallet(walletId: AptosWalletId = "petra") {
  // handle okx wallet special case since it's nested
  if (walletId === "okxwallet") {
    if (Object.hasOwn(window, "okxwallet") && Object.hasOwn(window.okxwallet, "aptos")) {
      return window.okxwallet.aptos
    }
  }
  if (Object.hasOwn(window, walletId)) return window[walletId] as AptosBrowserWallet
  if (Object.hasOwn(window, "aptos")) return window.aptos

  window.open("https://petra.app/", "_blank", "noopener noreferrer")
}

export function createAptosStore(
  previousState: ChainWalletStore<"aptos"> & {
    connectedWallet: AptosWalletId | undefined
  } = {
    chain: "aptos",
    hoverState: "none",
    address: undefined,
    connectedWallet: undefined,
    connectionStatus: "disconnected"
  }
) {
  const { subscribe, set, update, reset } = persisted("aptos-store", previousState, {
    syncTabs: true,
    storage: "session"
  })

  return {
    set,
    update,
    subscribe,
    connect: async (walletId: string) => {
      if (walletId !== "okxwallet" && walletId !== "petra" && walletId !== "martian") return
      update(v => ({ ...v, connectionStatus: "connecting", connectedWallet: walletId }))
      const wallet = getAptosWallet(walletId)

      if (!wallet) {
        const walletInfo = aptosWalletsInformation.find(wallet => wallet.id === walletId)
        if (walletInfo) {
          const { deepLink, download } = walletInfo
          window.open(deepLink || download, "_blank", "noopener noreferrer")
        } else {
          window.open("https://petra.app/", "_blank", "noopener noreferrer")
        }
        return
      }

      const account = await wallet.connect()

      update(v => ({
        ...v,
        address: account?.address as Hex,
        connectedWallet: walletId,
        connectionStatus: account?.address ? "connected" : "disconnected"
      }))
    },
    disconnect: async () => {
      const aptosWallet = get({ subscribe })
      const wallet = getAptosWallet(aptosWallet.connectedWallet)
      console.info(`[aptos] aptosDisconnectClick`, get(aptosStore))

      const isConnected = await wallet?.isConnected()
      if (isConnected || aptosWallet.connectionStatus !== "disconnected") {
        await wallet?.disconnect()

        update(_ => ({
          chain: "aptos",
          hoverState: "none",
          address: undefined,
          connectedWallet: undefined,
          connectionStatus: "disconnected"
        }))
      }
    }
  }
}

export const aptosStore = createAptosStore()

aptosStore.subscribe(async _ => {
  //
})

export const userAddressAptos = derived([aptosStore], ([$aptosStore]) => {
  if (!$aptosStore?.address) return null

  return {
    canonical: $aptosStore.address,
    bytes: hexToBytes($aptosStore.address)
  }
})
