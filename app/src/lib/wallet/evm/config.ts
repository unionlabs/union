import "$lib/polyfill.ts"
import {
  http,
  fallback,
  webSocket,
  reconnect,
  serialize,
  getAccount,
  deserialize,
  watchAccount,
  createConfig,
  unstable_connector,
  connect as _connect,
  disconnect as _disconnect,
  type GetAccountReturnType,
  switchChain as _switchChain,
  createStorage as createWagmiStorage
} from "@wagmi/core"
import { createClient } from "viem"
import { writable } from "svelte/store"
import { sleep } from "$lib/utilities"
import { sepolia } from "@wagmi/core/chains"
import type { ChainWalletStore } from "$lib/wallet/types"
import { walletConnect, injected } from "@wagmi/connectors"
import { walletActionsEip5792, walletActionsEip3074 } from "viem/experimental"

const WALLET_CONNECT_PROJECT_ID = "49fe74ca5ded7142adefc69a7788d14a"
const ankrId = "bced07c1a0ee36409ee84dae4e4f65a25b57715ddd8f3f2fd261f2a6b5508505"

const chains = [sepolia] as const
export type ConfiguredChainId = (typeof chains)[number]["id"]

export type Wallet = GetAccountReturnType
export type ConnectedWallet = Wallet & { status: "connected" }

export type ConnectorType = "injected" | "walletConnect"

export const config = createConfig({
  chains: [sepolia],
  client: ({ chain }) => {
    return createClient({
      chain,
      cacheTime: 4_000,
      pollingInterval: 4_000,
      batch: { multicall: true },
      transport: fallback([
        unstable_connector(injected),
        http(`https://rpc.ankr.com/eth_sepolia/${ankrId}`),
        webSocket(`wss://rpc.ankr.com/eth_sepolia/ws/${ankrId}`)
      ])
    })
      .extend(walletActionsEip5792())
      .extend(walletActionsEip3074())
  },
  syncConnectedChain: true,
  multiInjectedProviderDiscovery: true,
  storage: createWagmiStorage({
    serialize,
    deserialize,
    key: "wagmi",
    storage: typeof window !== "undefined" ? window.sessionStorage : undefined
  }),
  connectors: [
    injected({ shimDisconnect: true, unstable_shimAsyncInject: 2_500 }),
    walletConnect({
      projectId: WALLET_CONNECT_PROJECT_ID,
      qrModalOptions: {
        enableExplorer: true
      },
      metadata: {
        name: "Union App",
        description: "Union App",
        url: "https://app.union.build",
        icons: []
      }
    })
  ]
})

export function createSepoliaStore(
  previousState: ChainWalletStore<"evm"> = {
    chain: "sepolia",
    hoverState: "none",
    address: getAccount(config).address,
    connectionStatus: getAccount(config).status,
    connectedWallet: getAccount(config).connector?.id
  }
) {
  console.log("[sepoliaStore] previousState", previousState)
  const { subscribe, set, update } = writable(previousState)
  return {
    set,
    update,
    subscribe,
    connect: async (walletId: EvmWalletId) => {
      console.log("[evm] connect --", { walletId })
      await evmConnect(walletId, sepolia.id)
      await sleep(1_000)
    },
    disconnect: async () => {
      console.log("[evm] disconnect")
      await Promise.all([
        await evmDisconnect(),
        ...config.connectors.map(connector => connector.disconnect())
      ])
      await sleep(1_000)
    }
  }
}

export const sepoliaStore = createSepoliaStore()

const desiredWalletIds = [
  "injected",
  "io.metamask",
  "app.phantom",
  "io.tokenary",
  "walletconnect",
  "io.metamask.flask"
]

export const evmWalletsInformation = config.connectors
  .map(connector => ({
    ...connector,
    name: connector.name.toLowerCase().includes("injected") ? "Browser Wallet" : connector.name,
    icon: connector.id.toLowerCase().includes("walletconnect")
      ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23f7f7f7' d='M4.91 7.52a10.18 10.18 0 0 1 14.18 0l.47.46a.48.48 0 0 1 0 .7l-1.61 1.57a.25.25 0 0 1-.36 0l-.65-.63a7.1 7.1 0 0 0-9.88 0l-.7.68a.26.26 0 0 1-.35 0L4.4 8.72a.48.48 0 0 1 0-.7zm17.5 3.26 1.44 1.4a.48.48 0 0 1 0 .7l-6.46 6.33a.51.51 0 0 1-.71 0l-4.59-4.5a.13.13 0 0 0-.18 0l-4.59 4.5a.51.51 0 0 1-.7 0L.14 12.88a.48.48 0 0 1 0-.7l1.43-1.4a.51.51 0 0 1 .71 0l4.59 4.5c.05.04.13.04.18 0l4.59-4.5a.51.51 0 0 1 .7 0l4.6 4.5c.04.04.12.04.17 0l4.6-4.5a.5.5 0 0 1 .7 0'/%3E%3C/svg%3E"
      : connector.name.toLowerCase().includes("injected") || connector.icon === undefined
        ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 32 32'%3E%3Cpath fill='%23f7f7f7' fill-rule='evenodd' d='m15.65 3.64-9.6 4.8 10.2 5.1 10.2-5.1-9.6-4.8a1.35 1.35 0 0 0-1.2 0ZM28 10.46l-10.5 5.25v12.81l9.75-4.87a1.35 1.35 0 0 0 .75-1.21V10.46ZM15 28.53V15.7L4.5 10.46v11.97a1.35 1.35 0 0 0 .74 1.22L15 28.53Zm-.48 2.55-10.4-5.2A3.85 3.85 0 0 1 2 22.42V10.05A3.85 3.85 0 0 1 4.13 6.6l10.4-5.2a3.85 3.85 0 0 1 3.43 0l10.4 5.2a3.85 3.85 0 0 1 2.14 3.45v12.39a3.85 3.85 0 0 1-2.13 3.44l-10.4 5.2a3.85 3.85 0 0 1-3.45 0Z' clip-rule='evenodd'/%3E%3C/svg%3E"
        : connector.icon,
    type: connector.type as ConnectorType,
    download: ""
  }))
  .filter(connector => desiredWalletIds.includes(connector.id.toLowerCase()))

export type EvmWalletName = (typeof evmWalletsInformation)[number]["name"]
export type EvmWalletId = (typeof evmWalletsInformation)[number]["id"]

watchAccount(config, {
  onChange: account =>
    sepoliaStore.set({
      chain: "sepolia",
      hoverState: "none",
      address: account.address,
      connectionStatus: account.status,
      connectedWallet: account.connector?.id
    })
})
reconnect(config)

export async function evmConnect(
  evmWalletId: EvmWalletId,
  chainId: ConfiguredChainId = sepolia.id
) {
  const connectors = config.connectors.filter(connector => connector.id === evmWalletId)
  const connector = connectors[0] ?? connectors[1]
  if (connector) return _connect(config, { connector, chainId })
}

export function evmDisconnect() {
  return _disconnect(config, { connector: getAccount(config).connector })
}

export const evmSwitchChain = (chainId: ConfiguredChainId) => _switchChain(config, { chainId })
