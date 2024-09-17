import type { Address } from "viem"
import { bytesToHex } from "@unionlabs/client"
import { sleep } from "$lib/utilities/index.ts"
import { persisted } from "svelte-persisted-store"
import type { UserAddressCosmos } from "$lib/types"
import type { OfflineSigner } from "@leapwallet/types"
import type { ChainWalletStore } from "$lib/wallet/types"
import { derived, get, type Readable } from "svelte/store"
import { unionKeplrChainInfo, unionLeapChainInfo } from "$lib/wallet/cosmos/chain-info.ts"

export const cosmosWalletsInformation = [
  {
    id: "leap",
    name: "leap",
    icon: "/images/icons/leap.svg",
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
      console.info(walletId)
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
          connectedWallet: undefined,
          connectionStatus: "disconnected",
          address: undefined,
          rawAddress: undefined
        }))
      }
    }
  }
}

export const cosmosStore = createCosmosStore()

export const getCosmosOfflineSigner = (chainId: string): Promise<OfflineSigner> =>
  // @ts-expect-error
  get(cosmosStore).connectedWallet === "keplr"
    ? window.keplr?.getOfflineSignerAuto(chainId, { disableBalanceCheck: false })
    : window.leap?.getOfflineSignerAuto(chainId, { disableBalanceCheck: false })


export const userAddrCosmos: Readable<UserAddressCosmos | null> = derived(
  [cosmosStore],
  ([$cosmosStore]) => {
    if ($cosmosStore?.rawAddress && $cosmosStore?.address) {
      const cosmos_normalized = bytesToHex($cosmosStore.rawAddress)
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
