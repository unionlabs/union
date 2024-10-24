import { get } from "svelte/store"
import type { State } from "@wagmi/core"
import { persisted } from "svelte-persisted-store"
import type { ChainWalletStore } from "../types.ts"

/**
 * TODO:
 *
 * - check with Petra wallet team for proper `window.aptos` types
 */

export function getAptosWallet() {
  if (Object.hasOwn(window, "aptos")) return window.aptos
  if (Object.hasOwn(window, "petra")) return window.petra

  window.open("https://petra.app/", "_blank", "noopener noreferrer")
}

export const aptosWalletsInformation = [
  {
    id: "petra",
    name: "Petra",
    icon: "/images/icons/petra.svg",
    deepLink: "https://petra.app",
    download: "https://petra.app"
  }
] as const

export type AptosWalletId = (typeof aptosWalletsInformation)[number]["id"]

export function createAptosStore(
  previousState: ChainWalletStore<"aptos"> & {
    connectedWallet: AptosWalletId | undefined
  } = {
    chain: "aptos",
    hoverState: "none",
    address: undefined,
    connectedWallet: "petra",
    connectionStatus: "disconnected"
  }
) {
  const walletCore = getAptosWallet()

  const isConnected = (() => walletCore?.isConnected())()

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

      const account = await walletCore.connect()

      update(v => ({
        ...v,
        address: account?.address,
        connectedWallet: "petra",
        connectionStatus: account?.address ? "connected" : "disconnected"
      }))
    },
    disconnect: async () => {
      const aptosWalletId = get({ subscribe }).connectedWallet as AptosWalletId
      const walletCore = getAptosWallet()
      console.info(`[aptos] aptosDisconnectClick`, get(aptosStore))

      const isConnected = await walletCore?.isConnected()
      if (isConnected) {
        await walletCore?.disconnect()

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
