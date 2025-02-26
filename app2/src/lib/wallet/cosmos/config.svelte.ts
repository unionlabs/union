import { type OfflineSigner, bech32AddressToHex } from "@unionlabs/client"
import { unionKeplrChainInfo, unionLeapChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { Effect, Option } from "effect"
import { AddressCosmosCanonical } from "$lib/schema/address.ts"

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

  // Set up the derived calculation as a class field
  addressMapping = $derived(() => {
    if (this.rawAddress && this.address) {
      const cosmosAddressFromBech32 = (
        bech32Address: string
      ): typeof AddressCosmosCanonical.Type => {
        const hexAddress = bech32AddressToHex({ address: bech32Address })
        return AddressCosmosCanonical.make(hexAddress)
      }

      wallets.cosmosAddress = Option.some(cosmosAddressFromBech32(this.address))
    } else {
      wallets.cosmosAddress = Option.none()
    }
  })

  constructor() {
    // Initialize from session storage if available
    this.loadFromStorage()
  }

  loadFromStorage() {
    try {
      const storedData = sessionStorage.getItem("cosmos-store")
      if (storedData) {
        const parsedData = JSON.parse(storedData)
        this.chain = parsedData.chain || "cosmos"
        this.address = parsedData.address
        this.rawAddress = parsedData.rawAddress
          ? new Uint8Array(Object.values(parsedData.rawAddress))
          : undefined
        this.connectedWallet = parsedData.connectedWallet
        this.connectionStatus = parsedData.connectionStatus || "disconnected"
      }
    } catch (e) {
      console.error("Failed to load cosmos store from session storage", e)
    }
  }

  saveToStorage() {
    try {
      const storeData = {
        chain: this.chain,
        address: this.address,
        rawAddress: this.rawAddress,
        connectedWallet: this.connectedWallet,
        connectionStatus: this.connectionStatus
      }
      sessionStorage.setItem("cosmos-store", JSON.stringify(storeData))
    } catch (e) {
      console.error("Failed to save cosmos store to session storage", e)
    }
  }

  async connect(walletId: string) {
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
      await walletApi.enable(["union-testnet-9"])
    } catch (e) {
      console.log(e)
      this.connectionStatus = "disconnected"
      this.saveToStorage()
      return
    }

    const account = await walletApi.getKey("union-testnet-9")
    this.connectionStatus = "connected"
    this.address = account?.bech32Address
    this.rawAddress = account?.address
    this.connectedWallet = cosmosWalletId
    this.saveToStorage()

    await Effect.sleep(2_000)
  }

  async disconnect() {
    const cosmosWalletId = this.connectedWallet as CosmosWalletId
    console.log("[cosmos] cosmosDisconnectClick", this)

    if (cosmosWalletId && window[cosmosWalletId]) {
      if (cosmosWalletId === "keplr") {
        await window[cosmosWalletId]?.disable("union-testnet-9")
      } else if (cosmosWalletId === "leap") {
        await window[cosmosWalletId]?.disconnect("union-testnet-9")
      }

      this.connectedWallet = undefined
      this.connectionStatus = "disconnected"
      this.address = undefined
      this.rawAddress = undefined

      sessionStorage.removeItem("cosmos-store")
    }
  }
}

export const cosmosStore = new CosmosStore()

export const getCosmosOfflineSigner = ({
  chainId,
  connectedWallet
}: {
  chainId: string
  connectedWallet: CosmosWalletId
}): Promise<OfflineSigner> => {
  const signer = window[connectedWallet]?.getOfflineSignerAuto(chainId, {
    disableBalanceCheck: false
  })

  if (!signer) {
    return Promise.reject(new Error(`Failed to get offline signer for ${connectedWallet}`))
  }

  return signer
}
