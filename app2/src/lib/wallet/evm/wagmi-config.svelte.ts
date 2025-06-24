import { uiStore } from "$lib/stores/ui.svelte.ts"
import type { Edition } from "$lib/themes"
import { coinbaseWallet, injected, metaMask, safe, walletConnect } from "@wagmi/connectors"
import {
  createConfig,
  createStorage as createWagmiStorage,
  deserialize,
  fallback,
  http,
  serialize,
  unstable_connector,
} from "@wagmi/core"
import {
  arbitrumSepolia,
  berachainTestnetbArtio,
  bob,
  bobSepolia,
  bsc,
  bscTestnet,
  corn,
  cornTestnet,
  holesky,
  mainnet,
  scrollSepolia,
  sei,
  seiDevnet,
  seiTestnet,
  sepolia,
} from "@wagmi/core/chains"

export const chains = [
  arbitrumSepolia,
  berachainTestnetbArtio,
  bob,
  bobSepolia,
  bsc,
  bscTestnet,
  corn,
  cornTestnet,
  holesky,
  mainnet,
  scrollSepolia,
  sei,
  seiTestnet,
  sepolia,
] as const

export function getChainFromWagmi(chainId: number) {
  return chains.find((chain) => chain.id === chainId)
}

let wagmiConfig = createWagmiConfigInstance()

function createWagmiConfigInstance() {
  const edition: Edition = uiStore.edition

  return createConfig({
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
          name: "unstable_connector-injected-mainnet",
        }),
        http(`https://rpc.1.ethereum.chain.kitchen`, {
          name: "Chain Kitchen - Mainnet",
        }),
        http(sepolia.rpcUrls.default.http.at(0), {
          name: "default Mainnet RPC",
        }),
      ]),
      [sepolia.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-sepolia",
          name: "unstable_connector-injected-sepolia",
        }),
        http(`https://rpc.11155111.sepolia.chain.kitchen`, {
          name: "Chain Kitchen - Sepolia",
        }),
        http(sepolia.rpcUrls.default.http.at(0), {
          name: "default Sepolia RPC",
        }),
      ]),
      [holesky.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-holesky",
          name: "unstable_connector-injected-holesky",
        }),
        http(`https://rpc.17000.holesky.chain.kitchen`, {
          name: "Chain Kitchen - Holesky",
        }),
        http(holesky.rpcUrls.default.http.at(0), {
          name: "default Holesky RPC",
        }),
      ]),
      [berachainTestnetbArtio.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-berachain",
          name: "unstable_connector-injected-berachain",
        }),
        http(berachainTestnetbArtio.rpcUrls.default.http.at(0), {
          name: "default Berachain RPC",
        }),
      ]),
      [arbitrumSepolia.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-arbitrum-sepolia",
          name: "unstable_connector-injected-arbitrum-sepolia",
        }),
        http(arbitrumSepolia.rpcUrls.default.http.at(0), {
          name: "default Arbitrum Sepolia RPC",
        }),
      ]),
      [scrollSepolia.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-scroll-sepolia",
          name: "unstable_connector-injected-scroll-sepolia",
        }),
        http(scrollSepolia.rpcUrls.default.http.at(0), {
          name: "default Scroll Sepolia RPC",
        }),
      ]),
      [bobSepolia.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-bob-sepolia",
          name: "unstable_connector-injected-bob-sepolia",
        }),
        http(bobSepolia.rpcUrls.default.http.at(0), {
          name: "default Bob Sepolia RPC",
        }),
      ]),
      [bob.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-bob",
          name: "unstable_connector-injected-bob",
        }),
        http(bob.rpcUrls.default.http.at(0), { name: "default Bob RPC" }),
      ]),
      [corn.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-corn",
          name: "unstable_connector-injected-corn",
        }),
        http(corn.rpcUrls.default.http.at(0), { name: "default Corn RPC" }),
      ]),
      [cornTestnet.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-corn-testnet",
          name: "unstable_connector-injected-corn-testnet",
        }),
        http(cornTestnet.rpcUrls.default.http.at(0), {
          name: "default Corn Testnet RPC",
        }),
      ]),
      [sei.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-sei",
          name: "unstable_connector-injected-sei",
        }),
        http(sei.rpcUrls.default.http.at(0), { name: "default Sei RPC" }),
      ]),
      [seiTestnet.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-sei-testnet",
          name: "unstable_connector-injected-sei-testnet",
        }),
        http(seiTestnet.rpcUrls.default.http.at(0), { name: "default Sei Testnet RPC" }),
      ]),
      [bsc.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-bsc",
          name: "unstable_connector-injected-bsc",
        }),
        http(bsc.rpcUrls.default.http.at(0), { name: "default BSC RPC" }),
      ]),
      [bscTestnet.id]: fallback([
        unstable_connector(injected, {
          retryCount: 3,
          retryDelay: 100,
          key: "unstable_connector-injected-bsc-testnet",
          name: "unstable_connector-injected-bsc-testnet",
        }),
        http(bscTestnet.rpcUrls.default.http.at(0), { name: "default BSC Testnet RPC" }),
      ]),
    },
    storage: createWagmiStorage({
      serialize,
      deserialize,
      key: `union-wagmi-${edition}`,
      storage: typeof window !== "undefined" ? window.localStorage : undefined,
    }),
    connectors: [
      injected({
        shimDisconnect: true,
        unstable_shimAsyncInject: 2_500,
      }),
      coinbaseWallet({
        darkMode: true,
        appName: uiStore.appInfo.name,
        appLogoUrl: uiStore.appInfo.iconUrl,
        enableMobileWalletLink: true,
      }),
      metaMask({
        injectProvider: true,
        dappMetadata: {
          name: uiStore.appInfo.name,
          url: uiStore.appInfo.baseUrl,
          iconUrl: uiStore.appInfo.iconUrl,
        },
        useDeeplink: true,
      }),
      safe({
        allowedDomains: [
          /gnosis-safe.io$/,
          /app.safe.global$/,
          /staging.btc.union.build$/,
          /btc.union.build$/,
        ],
      }),
      walletConnect({
        projectId: uiStore.appInfo.projectId,
        showQrModal: true,
        metadata: {
          name: uiStore.appInfo.name,
          description: "Connect via WalletConnect",
          url: uiStore.appInfo.baseUrl,
          icons: [uiStore.appInfo.iconUrl],
        },
      }),
    ],
  })
}

export function getWagmiConfig() {
  return wagmiConfig
}

export type ConfiguredChainId = (typeof chains)[number]["id"]
