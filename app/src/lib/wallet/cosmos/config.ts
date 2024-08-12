import { derived, get, type Readable } from "svelte/store"
import { sleep } from "$lib/utilities/index.ts"
import { persisted } from "svelte-persisted-store"
import type { ChainWalletStore } from "$lib/wallet/types"
import { unionKeplrChainInfo, unionLeapChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import type { UserAddressCosmos } from "$lib/types"
import { rawToHex } from "$lib/utilities/address"
import type { Address } from "viem"

export const cosmosWalletsInformation = [
  {
    id: "leap",
    name: "leap",
    icon: "https://assets.leapwallet.io/logos/leap-cosmos-logo.svg",
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
    icon: "https://assets-global.website-files.com/63eb7ddf41cf5b1c8fdfbc74/63fc1eaf76d6a3bd547b017c_Keplr_icon_ver.1.3_2.svg",
    deepLink:
      "https://deeplink.keplr.app?url=app.union.build#Intent;package=com.chainapsis.keplr;scheme=keplrwallet;end;",
    download: "https://keplr.app/download"
  }
] as const

export type CosmosWalletId = (typeof cosmosWalletsInformation)[number]["id"]

function createCosmosStore(
  previousState: ChainWalletStore<"cosmos"> = {
    chain: "cosmos",
    hoverState: "none",
    address: undefined,
    rawAddress: undefined,
    connectedWallet: "keplr",
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
    connect: async (walletId: string) => {
      if (!walletId || (walletId !== "keplr" && walletId !== "leap")) return
      update(v => ({ ...v, connectionStatus: "connecting", connectedWallet: walletId }))
      const walletApi = window[walletId]
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
        await walletApi.enable(["union-testnet-8"])
      } catch (e) {
        return update(v => ({ ...v, connectionStatus: "disconnected" }))
      }
      const account = await walletApi.getKey("union-testnet-8")
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
          await window[cosmosWalletId]?.disable("union-testnet-8")
        } else if (cosmosWalletId === "leap") {
          await window[cosmosWalletId]?.disconnect("union-testnet-8")
        }
        update(v => ({
          ...v,
          connectedWallet: "none",
          connectionStatus: "disconnected",
          address: undefined,
          rawAddress: undefined
        }))
      }
    }
  }
}

export const cosmosStore = createCosmosStore()

export const userAddrCosmos: Readable<UserAddressCosmos | null> = derived(
  [cosmosStore],
  ([$cosmosStore]) => {
    if ($cosmosStore?.rawAddress && $cosmosStore?.address) {
      const cosmos_normalized = rawToHex($cosmosStore.rawAddress)
      return {
        canonical: $cosmosStore.address,
        normalized: cosmos_normalized,
        bytes: $cosmosStore.rawAddress,
        normalized_prefixed: `0x${cosmos_normalized}` as Address
      }
    }

    return null
  }
)
