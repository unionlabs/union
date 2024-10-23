// import {
//   WalletCore,
//   convertNetwork,
//   getAptosConfig,
//   isAptosNetwork,
//   isAptosConnectWallet,
//   getAptosConnectWallets
// } from "@aptos-labs/wallet-adapter-core"
import { get } from "svelte/store"
import type { State } from "@wagmi/core"
import { persisted } from "svelte-persisted-store"
import type { ChainWalletStore } from "$lib/wallet/types"

/**
 * TODO:
 *
 * - check with Petra wallet team for proper `window.aptos` types
 */

export function getAptosWallet() {
  if (Object.hasOwn(window, "aptos")) {
    return window.aptos
  }
  window.open("https://petra.app/", "_blank", "noopener noreferrer")
}

export const aptosWalletsInformation = [
  {
    id: "petra",
    name: "Petra",
    icon: "/images/icons/petra.svg",
    deepLink: "",
    download: "https://petra.app/"
  }
] as const

export type AptosWalletId = (typeof aptosWalletsInformation)[number]["id"]

export function createAptosStore(
  previousState: ChainWalletStore<"aptos"> & {
    rawAddress: undefined
    connectedWallet: AptosWalletId | undefined
  } = {
    chain: "aptos",
    hoverState: "none",
    address: undefined,
    rawAddress: undefined,
    connectedWallet: "petra",
    connectionStatus: "disconnected"
  }
) {
  const walletCore = getAptosWallet()

  const isConnected = (() => {
    return walletCore?.isConnected()
  })()

  const { subscribe, set, update, reset } = persisted(
    "aptos-store",
    {
      ...previousState,
      connectionStatus: isConnected ? "connected" : ("disconnected" as State["status"])
    },
    {
      syncTabs: true,
      storage: "session"
    }
  )

  return {
    set,
    update,
    subscribe,
    connect: async (walletId: string) => {
      console.info(`[aptos] aptosConnectClick`, walletId)
      const walletCore = getAptosWallet()
      if (!walletCore) {
        window.open("https://petra.app/", "_blank", "noopener noreferrer")
        return
      }

      // @ts-expect-error
      const account = (await walletCore.connect()) as { address: string }

      update(v => ({
        ...v,
        address: account?.address,
        rawAddress: undefined,
        connectedWallet: "petra",
        connectionStatus: account?.address ? "connected" : "disconnected"
      }))
    },
    disconnect: async () => {
      const aptosWalletId = get({ subscribe }).connectedWallet as AptosWalletId
      const walletCore = getAptosWallet()
      console.info(aptosWalletId)
      console.info(`[aptos] aptosDisconnectClick`, get(aptosStore))

      const isConnected = await walletCore?.isConnected()
      if (isConnected) {
        await walletCore?.disconnect()

        update(v => ({
          address: undefined,
          rawAddress: undefined,
          connectedWallet: undefined,
          connectionStatus: "disconnected",
          hoverState: "none",
          chain: "aptos"
        }))
      }
    }
  }
}

export const aptosStore = createAptosStore()

aptosStore.subscribe(async v => {
  // console.info(`[aptos] aptosStore.subscribe`, await v)
})
