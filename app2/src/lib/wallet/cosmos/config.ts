import { sleep } from "$lib/utils"
import { persisted } from "svelte-persisted-store"
import type { ChainWalletStore } from "$lib/wallet/types"
import { get } from "svelte/store"
import { type OfflineSigner, bech32AddressToHex } from "@unionlabs/client"
import { unionKeplrChainInfo, unionLeapChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { Option } from "effect"
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

function createCosmosStore(
  previousState: ChainWalletStore<"cosmos"> & {
    rawAddress: Uint8Array | undefined
    connectedWallet: CosmosWalletId | undefined
  } = {
    chain: "cosmos",
    address: undefined,
    rawAddress: undefined,
    connectedWallet: undefined,
    connectionStatus: "disconnected"
  }
) {
  const { subscribe, set, update } = persisted("cosmos-store", previousState, {
    syncTabs: true,
    storage: "session"
  })
  return {
    set,
    update,
    subscribe,
    connect: async (walletId: "leap" | "keplr") => {
      if (!walletId || (walletId !== "keplr" && walletId !== "leap")) return
      update(v => ({ ...v, connectionStatus: "connecting", connectedWallet: walletId }))
      const walletApi = window[walletId]

      console.log("zkgm1")
      if (!walletApi) {
        const walletInfo = cosmosWalletsInformation.find(wallet => wallet.id === walletId)
        if (walletInfo) {
          const { deepLink, download } = walletInfo
          window.open(deepLink || download, "_blank", "noopener noreferrer")
        }
        return update(v => ({ ...v, connectionStatus: "disconnected" }))
      }
      const chainInfoMap = {
        keplr: unionKeplrChainInfo,
        leap: unionLeapChainInfo
      }
      const chainInfo = chainInfoMap[walletId]
      if (!chainInfo) {
        alert("Chain information is missing for the selected wallet.")
        return update(v => ({ ...v, connectionStatus: "disconnected" }))
      }
      try {
        await walletApi.experimentalSuggestChain(chainInfo)
        await walletApi.enable(["union-testnet-9"])
      } catch (e) {
        console.log(e)
        return update(v => ({ ...v, connectionStatus: "disconnected" }))
      }
      const account = await walletApi.getKey("union-testnet-9")
      update(v => ({
        ...v,
        connectionStatus: "connected",
        address: account?.bech32Address,
        rawAddress: account?.address,
        connectedWallet: walletId
      }))
      await sleep(2_000)
    },
    disconnect: async () => {
      const cosmosWalletId = get({ subscribe }).connectedWallet as CosmosWalletId
      console.log("[cosmos] cosmosDisconnectClick", get(cosmosStore))
      if (cosmosWalletId && window[cosmosWalletId]) {
        if (cosmosWalletId === "keplr") {
          await window[cosmosWalletId]?.disable("union-testnet-9")
        } else if (cosmosWalletId === "leap") {
          await window[cosmosWalletId]?.disconnect("union-testnet-9")
        }
        update(v => ({
          ...v,
          connectedWallet: undefined,
          connectionStatus: "disconnected",
          address: undefined,
          rawAddress: undefined
        }))

        // Add this line to ensure the session storage is completely cleared
        sessionStorage.removeItem("cosmos-store")
      }
    }
  }
}

export const cosmosStore = createCosmosStore()

export const getCosmosOfflineSigner = ({
  chainId,
  connectedWallet
}: {
  chainId: string
  connectedWallet: CosmosWalletId
}): Promise<OfflineSigner> =>
  //@ts-ignore
  window[connectedWallet]?.getOfflineSignerAuto(chainId, { disableBalanceCheck: false })

cosmosStore.subscribe($cosmosStore => {
  if ($cosmosStore?.rawAddress && $cosmosStore?.address) {
    const cosmosAddressFromBech32 = (bech32Address: string): typeof AddressCosmosCanonical.Type => {
      const hexAddress = bech32AddressToHex({ address: bech32Address })
      return AddressCosmosCanonical.make(hexAddress)
    }

    wallets.cosmosAddress = Option.some(cosmosAddressFromBech32($cosmosStore.address))
  } else {
    wallets.cosmosAddress = Option.none()
  }
})
