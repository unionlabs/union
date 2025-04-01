import type { Hex } from "viem"
import { Option } from "effect"

import type { AptosBrowserWallet } from "@unionlabs/client"
import { AddressAptosCanonical } from "@unionlabs/sdk/schema"
import { wallets } from "$lib/stores/wallets.svelte.ts"

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
      icon: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAJDSURBVHgB7Zq9jtpAEMfHlhEgQLiioXEkoAGECwoKxMcTRHmC5E3IoyRPkPAEkI7unJYmTgEFTYwA8a3NTKScLnCHN6c9r1e3P2llWQy7M/s1Gv1twCP0ej37dDq9x+Zut1t3t9vZjDEHIiSRSPg4ZpDL5fxkMvn1cDh8m0wmfugfO53OoFQq/crn8wxfY9EymQyrVCqMfHvScZx1p9ls3pFxXBy/bKlUipGPrVbLuQqAfsCliq3zl0H84zwtjQrOw4Mt1W63P5LvBm2d+Xz+YzqdgkqUy2WgWCy+Mc/nc282m4FqLBYL+3g8fjDxenq72WxANZbLJeA13zDX67UDioL5ybXwafMYu64Ltn3bdDweQ5R97fd7GyhBQMipx4POeEDHIu2LfDdBIGGz+hJ9CQ1ABjoA2egAZPM6AgiCAEQhsi/C4jHyPA/6/f5NG3Ks2+3CYDC4aTccDrn6ojG54MnEvG00GoVmWLIRNZ7wTCwDHYBsdACy0QHIhiuRETxlICWpMMhGZHmqS8qH6JLyGegAZKMDkI0uKf8X4SWlaZo+Pp1bRrwlJU8ZKLIvUjKh0WiQ3sRUbNVq9c5Ebew7KEo2m/1p4jJ4qAmDaqDQBzj5XyiAT4VCQezJigAU+IDU+z8vJFnGWeC+bKQV/5VZ71FV6L7PA3gg3tXrdQ+DgLhC+75Wq3no69P3MC0NFQpx2lL04Ql9gHK1bRDjsSBIvScBnDTk1WrlGIZBorIDEYJj+rhdgnQ67VmWRe0zlplXl81vcyEt0rSoYDUAAAAASUVORK5CYII=",
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
  if (!window) return
  if (
    walletId === "okxwallet" &&
    Object.hasOwn(window, "okxwallet") &&
    Object.hasOwn(window.okxwallet, "aptos")
  ) {
    return window.okxwallet.aptos
  }
  if (Object.hasOwn(window, walletId)) return window[walletId] as AptosBrowserWallet
  if (Object.hasOwn(window, "aptos")) return window.aptos

  window.open("https://petra.app/", "_blank", "noopener noreferrer")
}

class AptosStore {
  chain = $state("aptos")
  address = $state<Hex | undefined>(undefined)
  connectedWallet = $state<AptosWalletId | undefined>(undefined)
  connectionStatus = $state<"disconnected" | "connecting" | "connected">("disconnected")
  hoverState = $state<string>("none")

  // Set up the derived calculation as a class field
  constructor() {
    this.loadFromStorage()
  }

  updateAptosAddress = (hexAddress: Hex | undefined) => {
    if (hexAddress) {
      const aptosAddressFromHex = (address: string): typeof AddressAptosCanonical.Type => {
        const normalized = address.startsWith("0x") ? address : `0x${address}`
        return AddressAptosCanonical.make(normalized)
      }
      wallets.aptosAddress = Option.some(aptosAddressFromHex(hexAddress))
      console.log("Set wallets.aptosAddress to:", hexAddress)
    } else {
      wallets.aptosAddress = Option.none()
      console.log("Cleared wallets.aptosAddress")
    }
  }

  loadFromStorage = () => {
    try {
      const storedData = sessionStorage.getItem("aptos-store")
      if (storedData) {
        const parsedData = JSON.parse(storedData)
        this.chain = parsedData.chain || "aptos"
        this.address = parsedData.address
        this.connectedWallet = parsedData.connectedWallet
        this.connectionStatus = parsedData.connectionStatus || "disconnected"
        this.hoverState = parsedData.hoverState || "none"

        this.updateAptosAddress(this.address)
      }
    } catch (e) {
      console.error("Failed to load aptos store from session storage", e)
    }
  }

  saveToStorage = () => {
    try {
      const storeData = {
        chain: this.chain,
        address: this.address,
        connectedWallet: this.connectedWallet,
        connectionStatus: this.connectionStatus,
        hoverState: this.hoverState
      }
      sessionStorage.setItem("aptos-store", JSON.stringify(storeData))
    } catch (e) {
      console.error("Failed to save aptos store to session storage", e)
    }
  }

  connect = async (walletId: string) => {
    if (walletId !== "okxwallet" && walletId !== "petra" && walletId !== "martian") return

    this.connectionStatus = "connecting"
    this.connectedWallet = walletId as AptosWalletId
    this.saveToStorage()

    const wallet = getAptosWallet(walletId as AptosWalletId)

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

    this.address = account?.address as Hex
    this.connectedWallet = walletId as AptosWalletId
    this.connectionStatus = account?.address ? "connected" : "disconnected"

    this.updateAptosAddress(account?.address as Hex)

    this.saveToStorage()
  }

  disconnect = async () => {
    const walletId = this.connectedWallet
    const wallet = getAptosWallet(walletId)
    console.info(`[aptos] aptosDisconnectClick`, this)

    const isConnected = await wallet?.isConnected()
    if (isConnected || this.connectionStatus !== "disconnected") {
      await wallet?.disconnect()

      this.chain = "aptos"
      this.hoverState = "none"
      this.address = undefined
      this.connectedWallet = undefined
      this.connectionStatus = "disconnected"

      this.updateAptosAddress(undefined)

      sessionStorage.removeItem("aptos-store")
    }
  }

  updateAccount = (account: {
    chain?: string
    address?: Hex
    connectionStatus?: "disconnected" | "connecting" | "connected"
    connectedWallet?: AptosWalletId
    hoverState?: string
  }) => {
    if (account.chain) this.chain = account.chain

    if (account.address !== this.address) {
      this.address = account.address
      this.updateAptosAddress(account.address)
    }

    if (account.connectionStatus) this.connectionStatus = account.connectionStatus
    if (account.connectedWallet !== undefined) this.connectedWallet = account.connectedWallet
    if (account.hoverState) this.hoverState = account.hoverState
    this.saveToStorage()
  }
}

export const aptosStore = new AptosStore()
