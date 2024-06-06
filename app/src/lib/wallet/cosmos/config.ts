import {get} from "svelte/store"
import {sleep} from "$lib/utilities/index.ts"
import {persisted} from "svelte-persisted-store"
import type {ChainWalletStore} from "$lib/wallet/types"
import {keplrChainInfo, leapChainInfo} from "$lib/wallet/cosmos/chain-info.ts";

export const cosmosWalletsInformation = [
  {
    id: "leap",
    name: "leap",
    icon: "https://assets.leapwallet.io/logos/leap-cosmos-logo.svg",
    download: "https://www.leapwallet.io/download"
  },
  {
    id: "keplr",
    name: "keplr",
    icon: "https://assets-global.website-files.com/63eb7ddf41cf5b1c8fdfbc74/63fc1eaf76d6a3bd547b017c_Keplr_icon_ver.1.3_2.svg",
    download: "https://www.keplr.app/download"
  }
] as const

export type CosmosWalletId = (typeof cosmosWalletsInformation)[number]["id"]

const stored = localStorage.getItem("cosmos-config") || "{}"

function createCosmosStore(
  previousState: ChainWalletStore<"cosmos"> = {
    chain: "cosmos",
    hoverState: "none",
    address: undefined,
    connectedWallet: "keplr",
    connectionStatus: "disconnected"
  }
) {
  console.log("[cosmosStore] previousState", previousState)
  const {subscribe, set, update} = persisted("cosmos-store", previousState, {
    syncTabs: true,
    storage: "session"
  })
  return {
    set,
    update,
    subscribe,
    connect: async (walletId: string) => {
      if (!walletId || (walletId !== "keplr" && walletId !== "leap")) return
      update(v => ({...v, connectionStatus: "connecting", connectedWallet: walletId}))
      const walletApi = window[walletId];
      if (!walletApi) {
        alert(`Please install ${walletId} wallet`)
        return update(v => ({...v, connectionStatus: "disconnected"}))
      }
      const chainInfoMap = {
        keplr: keplrChainInfo,
        leap: leapChainInfo
      };
      const chainInfo = chainInfoMap[walletId];
      if (!chainInfo) {
        alert('Chain information is missing for the selected wallet.');
        return update(v => ({...v, connectionStatus: "disconnected"}));
      }
      try {
        await walletApi.experimentalSuggestChain(chainInfo);
        await walletApi.enable(["union-testnet-8"])
      } catch (e) {
        return update(v => ({...v, connectionStatus: "disconnected"}));
      }
      const account = await walletApi.getKey("union-testnet-8")
      update(v => ({
        ...v,
        connectionStatus: "connected",
        address: account?.bech32Address,
        connectedWallet: walletId
      }))
      await sleep(2_000)
    },
    disconnect: async () => {
      const cosmosWalletId = get({subscribe}).connectedWallet as CosmosWalletId
      console.log({cosmosWalletId})
      console.log("[cosmos] cosmosDisconnectClick", get(cosmosStore))
      if (cosmosWalletId && cosmosWalletId === "keplr" && window[cosmosWalletId]) {
        await window[cosmosWalletId]?.disable("union-testnet-8")
        update(v => ({...v, connectedWallet: "none", connectionStatus: "disconnected"}))
      }
      if (cosmosWalletId && cosmosWalletId === "leap" && window[cosmosWalletId]) {
        await window[cosmosWalletId]?.disconnect("union-testnet-8")
        update(v => ({...v, connectedWallet: "none", connectionStatus: "disconnected"}))
      }
    }
  }
}

export const cosmosStore = createCosmosStore()
// cosmosStore.subscribe(value => localStorage.setItem("cosmos-config", JSON.stringify(value)))

