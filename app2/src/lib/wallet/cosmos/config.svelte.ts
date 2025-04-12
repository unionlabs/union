import { bech32AddressToHex } from "@unionlabs/client"
import { unionKeplrChainInfo, unionLeapChainInfo } from "$lib/wallet/cosmos/chain-info"
import { wallets } from "$lib/stores/wallets.svelte"
import { Effect, Option } from "effect"
import { AddressCosmosCanonical } from "@unionlabs/sdk/schema"

export const cosmosWalletsInformation = [
  {
    id: "leap",
    name: "leap",
    icon: "/images/icons/leap.ico",
    /**
     * reference links:
     * - leap deep link generator: https://developers.leapwallet.io/deeplink-generator
     * - qr code: https://git-union69.web.val.run/app.union.build?svg=union.build/logo.svg&url=leapcosmoswallet.page.link/M3BmzUK5RRPsNyBe9?d=1
     */
    deepLink: "https://leapcosmoswallet.page.link/rXtQWTw1fSRuQCeZ8?d=1",
    download: "https://leapwallet.io/download"
  },
  /**
   * reference links:
   * - keplr link generator: https://chainapsis.notion.site/How-to-use-Deep-Link-on-Keplr-mobile-5593b89de65142278a6a7573c97ad0d4
   * - qr code: https://git-union69.web.val.run/app.union.build?svg=union.build/logo.svg&url=leapcosmoswallet.page.link/M3BmzUK5RRPsNyBe9?d=1
   */
  {
    id: "keplr",
    name: "keplr",
    icon: "/images/icons/keplr.svg",
    deepLink:
      "https://deeplink.keplr.app?url=app.union.build#Intent;package=com.chainapsis.keplr;scheme=keplrwallet;end;",
    download: "https://keplr.app/download"
  }
] as const

export type CosmosWalletId = (typeof cosmosWalletsInformation)[number]["id"]

class CosmosStore {
  chain = $state("cosmos")
  address = $state<string | undefined>(undefined)
  rawAddress = $state<Uint8Array | undefined>(undefined)
  connectedWallet = $state<CosmosWalletId | undefined>(undefined)
  connectionStatus = $state<"disconnected" | "connecting" | "connected">("disconnected")

  constructor() {
    this.loadFromStorage()

    if (this.connectedWallet && this.connectionStatus === "connected") {
      console.log("Attempting to auto-reconnect to Cosmos wallet:", this.connectedWallet)
      setTimeout(() => {
        this.reconnect(this.connectedWallet as CosmosWalletId)
      }, 1000)
    }
  }

  // Centralized method to update cosmos address
  updateCosmosAddress = (bech32Address: string | undefined) => {
    if (bech32Address) {
      const cosmosAddressFromBech32 = (address: string) => {
        const hexAddress = bech32AddressToHex({ address })
        return AddressCosmosCanonical.make(hexAddress)
      }
      wallets.cosmosAddress = Option.some(cosmosAddressFromBech32(bech32Address))
    } else {
      wallets.cosmosAddress = Option.none()
    }
  }

  loadFromStorage = () => {
    try {
      const storedData = sessionStorage.getItem("cosmos-store")

      if (storedData) {
        const parsedData = JSON.parse(storedData)

        this.chain = parsedData.chain || "cosmos"
        this.address = parsedData.address
        this.rawAddress = parsedData.rawAddress ? new Uint8Array(parsedData.rawAddress) : undefined
        this.connectedWallet = parsedData.connectedWallet
        this.connectionStatus = parsedData.connectionStatus || "disconnected"

        // Don't update wallets.cosmosAddress here if we're going to reconnect
        if (!(this.connectedWallet && this.connectionStatus === "connected")) {
          this.updateCosmosAddress(this.address)
        }

        console.log("Cosmos store loaded:", {
          chain: this.chain,
          address: this.address,
          hasRawAddress: !!this.rawAddress,
          connectedWallet: this.connectedWallet,
          connectionStatus: this.connectionStatus
        })
      }
    } catch (e) {
      console.error("Failed to load cosmos store from session storage", e)
    }
  }

  saveToStorage = () => {
    try {
      const storeData = {
        chain: this.chain,
        address: this.address,
        rawAddress: this.rawAddress ? Array.from(this.rawAddress) : undefined,
        connectedWallet: this.connectedWallet,
        connectionStatus: this.connectionStatus
      }
      sessionStorage.setItem("cosmos-store", JSON.stringify(storeData))
    } catch (e) {
      console.error("Failed to save cosmos store to session storage", e)
    }
  }

  connect = async (walletId: string) => {
    if (!walletId || (walletId !== "keplr" && walletId !== "leap")) return

    const cosmosWalletId = walletId as CosmosWalletId
    this.connectionStatus = "connecting"
    this.connectedWallet = cosmosWalletId
    this.saveToStorage()

    const walletApi = window[cosmosWalletId]
    if (!walletApi) {
      const walletInfo = cosmosWalletsInformation.find(wallet => wallet.id === cosmosWalletId)
      if (walletInfo) {
        const { deepLink, download } = walletInfo
        window.open(deepLink || download, "_blank", "noopener noreferrer")
      }
      this.connectionStatus = "disconnected"
      this.saveToStorage()
      return
    }

    const chainInfoMap = {
      keplr: unionKeplrChainInfo,
      leap: unionLeapChainInfo
    }

    const chainInfo = chainInfoMap[cosmosWalletId]
    if (!chainInfo) {
      alert("Chain information is missing for the selected wallet.")
      this.connectionStatus = "disconnected"
      this.saveToStorage()
      return
    }

    try {
      await walletApi.experimentalSuggestChain(chainInfo)
      await walletApi.enable(["bbn-1"])
    } catch (e) {
      console.log(e)
      this.connectionStatus = "disconnected"
      this.saveToStorage()
      return
    }

    const account = await walletApi.getKey("bbn-1")
    this.connectionStatus = "connected"
    this.address = account?.bech32Address
    this.rawAddress = account?.address
    this.connectedWallet = cosmosWalletId

    // Update wallets.cosmosAddress using the centralized method
    this.updateCosmosAddress(account?.bech32Address)

    this.saveToStorage()

    Effect.sleep(2_000)
  }

  reconnect = async (walletId: CosmosWalletId) => {
    if (!walletId) return

    const walletApi = window[walletId]
    if (!walletApi) {
      console.log("Wallet API not found for auto-reconnection")
      this.connectionStatus = "disconnected"
      this.saveToStorage()
      return
    }

    const chainInfoMap = {
      keplr: unionKeplrChainInfo,
      leap: unionLeapChainInfo
    }

    const chainInfo = chainInfoMap[walletId]
    if (!chainInfo) {
      console.error("Chain information is missing for the selected wallet.")
      this.connectionStatus = "disconnected"
      this.saveToStorage()
      return
    }

    try {
      // Try to enable the chain
      await walletApi.enable(["bbn-1"])

      // Get account information
      const account = await walletApi.getKey("bbn-1")

      if (account?.bech32Address) {
        this.connectionStatus = "connected"
        this.address = account.bech32Address
        this.rawAddress = account.address

        // Update wallets.cosmosAddress using the centralized method
        this.updateCosmosAddress(account.bech32Address)
      } else {
        this.connectionStatus = "disconnected"
        this.updateCosmosAddress(undefined)
      }

      this.saveToStorage()
    } catch (e) {
      console.error("Failed to auto-reconnect to Cosmos wallet:", e)
      this.connectionStatus = "disconnected"
      this.updateCosmosAddress(undefined)
      this.saveToStorage()
    }
  }

  disconnect = async () => {
    const cosmosWalletId = this.connectedWallet as CosmosWalletId

    if (cosmosWalletId && window[cosmosWalletId]) {
      if (cosmosWalletId === "keplr") {
        await window[cosmosWalletId]?.disable("bbn-1")
      } else if (cosmosWalletId === "leap") {
        await window[cosmosWalletId]?.disconnect("bbn-1")
      }

      this.connectedWallet = undefined
      this.connectionStatus = "disconnected"
      this.address = undefined
      this.rawAddress = undefined

      // Update wallets.cosmosAddress using the centralized method
      this.updateCosmosAddress(undefined)

      sessionStorage.removeItem("cosmos-store")
    }
  }
}

export const cosmosStore = new CosmosStore()
