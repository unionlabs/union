import {
  http,
  fallback,
  reconnect,
  serialize,
  getAccount,
  deserialize,
  createConfig,
  watchAccount,
  type Connector,
  unstable_connector,
  connect as _connect,
  disconnect as _disconnect,
  type GetAccountReturnType,
  switchChain as _switchChain,
  createStorage as createWagmiStorage
} from "@wagmi/core"
import type { Address } from "viem"
import { sleep } from "$lib/utilities"
import type { UserAddressEvm } from "$lib/types"
import { APP_INFO } from "$lib/constants/app.ts"
import type { ChainWalletStore } from "$lib/wallet/types"
import { derived, writable, type Readable } from "svelte/store"
import { injected, coinbaseWallet } from "@wagmi/connectors"
import {
  sepolia,
  holesky,
  berachainTestnetbArtio,
  arbitrumSepolia,
  scrollSepolia
} from "@wagmi/core/chains"

export const chains = [
  sepolia,
  holesky,
  berachainTestnetbArtio,
  arbitrumSepolia,
  scrollSepolia
] as const
export type ConfiguredChainId = (typeof chains)[number]["id"]

export type Wallet = GetAccountReturnType
export type ConnectedWallet = Wallet & { status: "connected" }

export type ConnectorType = "injected" | "walletConnect"

const WALLETCONNECT_PROJECT_ID = "49fe74ca5ded7142adefc69a7788d14a"

export const config = createConfig({
  chains,
  cacheTime: 4_000,
  pollingInterval: 4_000,
  syncConnectedChain: true,
  batch: { multicall: true },
  multiInjectedProviderDiscovery: true,
  transports: {
    [sepolia.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-sepolia",
        name: "unstable_connector-injected-sepolia"
      }),
      http(`https://rpc.11155111.sepolia.chain.kitchen`, {
        name: "Chain Kitchen - Sepolia"
      }),
      http(sepolia.rpcUrls.default.http.at(0), { name: "default Sepolia RPC" })
    ]),
    [holesky.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-holesky",
        name: "unstable_connector-injected-holesky"
      }),
      http(`https://rpc.17000.holesky.chain.kitchen`, {
        name: "Chain Kitchen - Holesky"
      }),
      http(holesky.rpcUrls.default.http.at(0), { name: "default Holesky RPC" })
    ]),
    [berachainTestnetbArtio.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-berachain",
        name: "unstable_connector-injected-berachain"
      }),
      http(berachainTestnetbArtio.rpcUrls.default.http.at(0), { name: "default Berachain RPC" })
    ]),
    [arbitrumSepolia.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-berachain",
        name: "unstable_connector-injected-berachain"
      }),
      http(arbitrumSepolia.rpcUrls.default.http.at(0), { name: "default Arbitrum Sepolia RPC" })
    ]),
    [scrollSepolia.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-scroll-sepolia",
        name: "unstable_connector-injected-scroll-sepolia"
      }),
      http(scrollSepolia.rpcUrls.default.http.at(0), { name: "default Scroll Sepolia RPC" })
    ])
  },
  storage: createWagmiStorage({
    serialize,
    deserialize,
    key: "union-wagmi",
    storage: typeof window !== "undefined" ? window.localStorage : undefined
  }),
  connectors: [
    injected({
      shimDisconnect: true,
      unstable_shimAsyncInject: 2_500,
      target() {
        return {
          id: "injected",
          name: "injected",
          provider: window.ethereum
        }
      }
    }),
    coinbaseWallet({
      darkMode: true,
      appName: APP_INFO.name,
      appLogoUrl: APP_INFO.iconUrl,
      enableMobileWalletLink: true
    })
  ]
})

// Persistent storage for last connected wallet
const LAST_CONNECTED_WALLET_KEY = "last-connected-wallet"
function getLastConnectedWalletId(): string | undefined {
  if (typeof window !== "undefined") {
    return window.localStorage.getItem(LAST_CONNECTED_WALLET_KEY) || undefined
  }
  return undefined
}

function setLastConnectedWalletId(walletId: string | undefined) {
  if (typeof window !== "undefined" && walletId) {
    window.localStorage.setItem(LAST_CONNECTED_WALLET_KEY, walletId)
  }
}

function clearLastConnectedWalletId() {
  if (typeof window !== "undefined") {
    window.localStorage.removeItem(LAST_CONNECTED_WALLET_KEY)
  }
}

export function createSepoliaStore(
  previousState: Omit<ChainWalletStore<"evm">, "rawAddress"> = {
    chain: "sepolia",
    hoverState: "none",
    address: undefined,
    connectionStatus: "disconnected",
    connectedWallet: getLastConnectedWalletId()
  }
) {
  const { subscribe, set, update } = writable(previousState)

  // Auto-reconnect to the last connected wallet on initialization
  const lastWalletId = getLastConnectedWalletId()
  if (lastWalletId) {
    const lastConnector = config.connectors.find(c => c.id === lastWalletId)
    if (lastConnector) {
      reconnect(config, { connectors: [lastConnector] })
        .then(() => {
          const account = getAccount(config)
          set({
            chain: "sepolia",
            hoverState: "none",
            address: account.address,
            connectionStatus: account.status,
            connectedWallet: account.connector?.id
          })
          startWatchingAccount() // Start watching after successful reconnect
        })
        .catch(error => {
          console.error("Auto-reconnection failed:", error)
          set({ ...previousState, connectionStatus: "disconnected" })
        })
    }
  }

  return {
    set,
    update,
    subscribe,
    connect: async (walletId: EvmWalletId) => {
      try {
        await evmConnect(walletId, sepolia.id)
        const account = getAccount(config)
        setLastConnectedWalletId(account.connector?.id)
        set({
          chain: "sepolia",
          hoverState: "none",
          address: account.address,
          connectionStatus: account.status,
          connectedWallet: account.connector?.id
        })
      } catch (error) {
        console.error("Connection failed:", error)
        set({ ...previousState, connectionStatus: "disconnected" })
      }
    },
    disconnect: async () => {
      try {
        await evmDisconnect()
        if (typeof window !== "undefined") {
          Object.keys(window.localStorage)
            .filter(key => key.startsWith("union-wagmi"))
            .forEach(key => window.localStorage.removeItem(key))
        }
        clearLastConnectedWalletId()
        set({
          chain: "sepolia",
          hoverState: "none",
          address: undefined,
          connectionStatus: "disconnected",
          connectedWallet: undefined
        })
      } catch (error) {
        console.error("Disconnect failed:", error)
      }
      await sleep(1_000)
    },
    reconnectLast: async () => {
      try {
        const lastWalletId = getLastConnectedWalletId()
        if (lastWalletId) {
          const lastConnector = config.connectors.find(c => c.id === lastWalletId)
          if (lastConnector) {
            await reconnect(config, { connectors: [lastConnector] })
            const account = getAccount(config)
            set({
              chain: "sepolia",
              hoverState: "none",
              address: account.address,
              connectionStatus: account.status,
              connectedWallet: account.connector?.id
            })
          } else {
            console.warn("Last connected connector not found:", lastWalletId)
            set({ ...previousState, connectionStatus: "disconnected" })
          }
        } else {
          console.log("No last connected wallet found")
          set({ ...previousState, connectionStatus: "disconnected" })
        }
      } catch (error) {
        console.error("Reconnection failed:", error)
        set({ ...previousState, connectionStatus: "disconnected" })
      }
    }
  }
}

export const sepoliaStore = createSepoliaStore()

export const userAddrEvm: Readable<UserAddressEvm | null> = derived(
  [sepoliaStore],
  ([$sepoliaStore]) => {
    if ($sepoliaStore?.address) {
      const evm_normalized = $sepoliaStore.address.slice(2).toLowerCase()
      return {
        canonical: $sepoliaStore.address as Address,
        normalized: evm_normalized,
        normalized_prefixed: `0x${evm_normalized}` as Address
      }
    }
    return null
  }
)

const excludeWalletList = ["io.leapwallet.LeapWallet", "app.keplr"]

export const evmWalletsInformation = config.connectors
  .map(connector => {
    const id = connector.id.toLowerCase()
    const name = connector.name.toLowerCase()
    return {
      ...connector,
      name: name.includes("injected") ? "Browser Wallet" : connector.name,
      icon: (id.includes("walletconnect")
        ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23268fff' d='M4.91 7.52a10.18 10.18 0 0 1 14.18 0l.47.46a.48.48 0 0 1 0 .7l-1.61 1.57a.25.25 0 0 1-.36 0l-.65-.63a7.1 7.1 0 0 0-9.88 0l-.7.68a.26.26 0 0 1-.35 0L4.4 8.72a.48.48 0 0 1 0-.7zm17.5 3.26 1.44 1.4a.48.48 0 0 1 0 .7l-6.46 6.33a.51.51 0 0 1-.71 0l-4.59-4.5a.13.13 0 0 0-.18 0l-4.59 4.5a.51.51 0 0 1-.7 0L.14 12.88a.48.48 0 0 1 0-.7l1.43-1.4a.51.51 0 0 1 .71 0l4.59 4.5c.05.04.13.04.18 0l4.59-4.5a.51.51 0 0 1 .7 0l4.6 4.5c.04.04.12.04.17 0l4.6-4.5a.5.5 0 0 1 .7 0' /%3E%3C/svg%3E%0A"
        : name.includes("injected")
          ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 32 32'%3E%3Cpath fill='%23f7f7f7' fill-rule='evenodd' d='m15.65 3.64-9.6 4.8 10.2 5.1 10.2-5.1-9.6-4.8a1.35 1.35 0 0 0-1.2 0ZM28 10.46l-10.5 5.25v12.81l9.75-4.87a1.35 1.35 0 0 0 .75-1.21V10.46ZM15 28.53V15.7L4.5 10.46v11.97a1.35 1.35 0 0 0 .74 1.22L15 28.53Zm-.48 2.55-10.4-5.2A3.85 3.85 0 0 1 2 22.42V10.05A3.85 3.85 0 0 1 4.13 6.6l10.4-5.2a3.85 3.85 0 0 1 3.43 0l10.4 5.2a3.85 3.85 0 0 1 2.14 3.45v12.39a3.85 3.85 0 0 1-2.13 3.44l-10.4 5.2a3.85 3.85 0 0 1-3.45 0Z' clip-rule='evenodd'/%3E%3C/svg%3E"
          : name.includes("coinbase")
            ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='56' height='56' fill='none'%3E%3Cpath d='M28 56c15.464 0 28-12.536 28-28S43.464 0 28 0 0 12.536 0 28s12.536 28 28 28Z' fill='%231B53E4'/%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M7 28c0 11.598 9.402 21 21 21s21-9.402 21-21S39.598 7 28 7 7 16.402 7 28Zm17.234-6.766a3 3 0 0 0-3 3v7.533a3 3 0 0 0 3 3h7.533a3 3 0 0 0 3-3v-7.533a3 3 0 0 0-3-3h-7.533Z' fill='%23fff'/%3E%3C/svg%3E"
            : connector.icon) as string,
      type: connector.type as ConnectorType,
      download: ""
    }
  })
  .filter(wallet => !excludeWalletList.includes(wallet.id)) satisfies Array<Connector>

export type EvmWalletId = (typeof evmWalletsInformation)[number]["id"]

let unwatch: (() => void) | undefined
export function startWatchingAccount() {
  if (!unwatch) {
    unwatch = watchAccount(config, {
      onChange: account => {
        sepoliaStore.set({
          chain: account.chain?.name ?? "sepolia",
          hoverState: "none",
          address: account.address,
          connectionStatus: account.status,
          connectedWallet: account.connector?.id
        })
      }
    })
  }
}

export async function evmConnect(
  evmWalletId: EvmWalletId,
  chainId: ConfiguredChainId = sepolia.id
) {
  const connector = config.connectors.find(connector => connector.id === evmWalletId)
  if (connector) {
    const result = await _connect(config, { connector, chainId })
    startWatchingAccount()
    return result
  }
  throw new Error(`Connector ${evmWalletId} not found`)
}

export async function evmDisconnect() {
  const connector = getAccount(config).connector
  if (connector) {
    await _disconnect(config, { connector })
  } else {
    await _disconnect(config)
  }
  if (unwatch) {
    unwatch()
    unwatch = undefined
  }
}

export const evmSwitchChain = (chainId: ConfiguredChainId) => _switchChain(config, { chainId })
