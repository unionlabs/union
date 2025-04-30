import {
  createConfig,
  createStorage as createWagmiStorage,
  deserialize,
  fallback,
  http,
  serialize,
  unstable_connector
} from "@wagmi/core"
import { coinbaseWallet, injected, metaMask, walletConnect, safe } from "@wagmi/connectors"
import {
  arbitrumSepolia,
  berachainTestnetbArtio,
  bob,
  bobSepolia,
  corn,
  cornTestnet,
  holesky,
  mainnet,
  scrollSepolia,
  sepolia
} from "@wagmi/core/chains"
import { uiStore } from "$lib/stores/ui.svelte.ts"

export const chains = [
  mainnet,
  corn,
  sepolia,
  holesky,
  berachainTestnetbArtio,
  arbitrumSepolia,
  scrollSepolia,
  bobSepolia,
  bob,
  cornTestnet
] as const

export function getChainFromWagmi(chainId: number) {
  return chains.find(chain => chain.id === chainId)
}

const WALLETCONNECT_PROJECT_ID = uiStore.appInfo.projectId

export const wagmiConfig = createConfig({
  chains,
  cacheTime: 4_000,
  pollingInterval: 4_000,
  syncConnectedChain: true,
  batch: { multicall: true },
  multiInjectedProviderDiscovery: true,
  transports: {
    [mainnet.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-mainnet",
        name: "unstable_connector-injected-mainnet"
      }),
      http(`https://rpc.1.ethereum.chain.kitchen`, {
        name: "Chain Kitchen - Mainnet"
      }),
      http(sepolia.rpcUrls.default.http.at(0), { name: "default Mainnet RPC" })
    ]),
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
        key: "unstable_connector-injected-arbitrum-sepolia",
        name: "unstable_connector-injected-arbitrum-sepolia"
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
    ]),
    [bobSepolia.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-bob-sepolia",
        name: "unstable_connector-injected-bob-sepolia"
      }),
      http(bobSepolia.rpcUrls.default.http.at(0), { name: "default Bob Sepolia RPC" })
    ]),
    [bob.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-bob",
        name: "unstable_connector-injected-bob"
      }),
      http(bob.rpcUrls.default.http.at(0), { name: "default Bob RPC" })
    ]),
    [corn.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-corn",
        name: "unstable_connector-injected-corn"
      }),
      http(bob.rpcUrls.default.http.at(0), { name: "default Corn RPC" })
    ]),
    [cornTestnet.id]: fallback([
      unstable_connector(injected, {
        retryCount: 3,
        retryDelay: 100,
        key: "unstable_connector-injected-corn-testnet",
        name: "unstable_connector-injected-corn-testnet"
      }),
      http(bob.rpcUrls.default.http.at(0), { name: "default Corn Testnet RPC" })
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
      unstable_shimAsyncInject: 2_500
    }),
    coinbaseWallet({
      darkMode: true,
      appName: uiStore.appInfo.name,
      appLogoUrl: uiStore.appInfo.iconUrl,
      enableMobileWalletLink: true
    }),
    metaMask({
      injectProvider: true,
      dappMetadata: {
        name: uiStore.appInfo.name,
        url: uiStore.appInfo.baseUrl,
        iconUrl: uiStore.appInfo.iconUrl
      },
      useDeeplink: true
    }),
    safe({
      allowedDomains: [
        /gnosis-safe.io$/,
        /app.safe.global$/,
        /staging.btc.union.build$/,
        /btc.union.build$/
      ]
    }),
    walletConnect({
      projectId: WALLETCONNECT_PROJECT_ID,
      showQrModal: true,
      metadata: {
        name: uiStore.appInfo.name,
        description: "Connect via WalletConnect",
        url: uiStore.appInfo.baseUrl,
        icons: [uiStore.appInfo.iconUrl]
      }
    })
  ]
})

wagmiConfig.subscribe(
  state => state.chainId,
  _chainId => {
    console.log("config state changed", _chainId)
  }
)

export type ConfiguredChainId = (typeof chains)[number]["id"]
