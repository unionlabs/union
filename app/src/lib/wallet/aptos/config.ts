import type { State } from "@wagmi/core"
import { derived, get } from "svelte/store"
import { hexToBytes, type Hex } from "viem"
import { persisted } from "svelte-persisted-store"
import type { ChainWalletStore } from "../types.ts"

/**
 * TODO:
 * - check with Petra wallet team for proper `window.aptos` types
 */

export function getAptosWallet() {
  if (Object.hasOwn(window, "petra")) return window.petra
  if (Object.hasOwn(window, "aptos")) return window.aptos

  window.open("https://petra.app/", "_blank", "noopener noreferrer")
}

export const aptosWalletsInformation = [
  {
    id: "petra",
    name: "Petra",
    icon: "/images/icons/petra.svg",
    download: "https://petra.app",
    deepLink: "https://petra.app/explore?link=https://app.union.build"
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
  const wallet = getAptosWallet()

  const isConnected = (() => wallet?.isConnected())()

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
      const wallet = getAptosWallet()
      if (!wallet) {
        window.open("https://petra.app/", "_blank", "noopener noreferrer")
        return
      }

      const account = await wallet.connect()

      update(v => ({
        ...v,
        address: account?.address as Hex,
        connectedWallet: "petra",
        connectionStatus: account?.address ? "connected" : "disconnected"
      }))
    },
    disconnect: async () => {
      const aptosWallet = get({ subscribe })
      const wallet = getAptosWallet()
      console.info(`[aptos] aptosDisconnectClick`, get(aptosStore))

      const isConnected = await wallet?.isConnected()
      if (isConnected || aptosWallet.connectionStatus !== 'disconnected') {
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
