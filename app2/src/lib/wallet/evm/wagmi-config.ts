import {
  createConfig,
  createStorage as createWagmiStorage,
  deserialize,
  fallback,
  http,
  serialize,
  unstable_connector
} from "@wagmi/core"
import {coinbaseWallet, injected, metaMask, walletConnect} from "@wagmi/connectors"
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
import { TESTNET_APP_INFO } from "$lib/config/app"

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

const WALLETCONNECT_PROJECT_ID = "49fe74ca5ded7142adefc69a7788d14a"

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
      appName: TESTNET_APP_INFO.name,
      appLogoUrl: TESTNET_APP_INFO.iconUrl,
      enableMobileWalletLink: true
    }),
    metaMask({
      injectProvider: true,
      dappMetadata: {
        name: TESTNET_APP_INFO.name,
        url: TESTNET_APP_INFO.baseUrl,
        iconUrl: TESTNET_APP_INFO.iconUrl
      },
      useDeeplink: true
    }),
    walletConnect({
      projectId: WALLETCONNECT_PROJECT_ID,
      showQrModal: true,
      metadata: {
        name: TESTNET_APP_INFO.name,
        description: "Connect via WalletConnect",
        url: TESTNET_APP_INFO.baseUrl,
        icons: [TESTNET_APP_INFO.iconUrl]
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
