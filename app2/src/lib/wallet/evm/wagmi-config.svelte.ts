import { uiStore } from "$lib/stores/ui.svelte"
import type { Edition } from "$lib/themes"
import { VIEM_CHAINS } from "@unionlabs/sdk/constants/viem-chains"
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
  seiTestnet,
  sepolia,
} from "@wagmi/core/chains"
import { Option } from "effect"
import type { Chain as ViemChain, FallbackTransport, Transport } from "viem"

let wagmiConfig: Option.Option<ReturnType<typeof createWagmiConfigInstance>> = Option.none()

type Transports = {
  [Item in (typeof VIEM_CHAINS)[number] as Item["id"]]: FallbackTransport<Transport[]>
}

const transports: Transports = {
  [mainnet.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-mainnet",
      name: "unstable_connector-injected-mainnet",
    }),
    http(`https://rpc.${mainnet.id}.ethereum.chain.kitchen`, {
      name: "Chain Kitchen - Ethereum",
    }),
    http(mainnet.rpcUrls.default.http.at(0), {
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
    http(`https://rpc.${sepolia.id}.ethereum.chain.kitchen`, {
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
    http(`https://rpc.${holesky.id}.ethereum.chain.kitchen`, {
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
    http(`https://rpc.${bobSepolia.id}.bob.chain.kitchen`, {
      name: "Chain Kitchen - BOB Sepolia",
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
    http(`https://rpc.${bob.id}.bob.chain.kitchen`, {
      name: "Chain Kitchen - BOB",
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
    http(`https://rpc.${corn.id}.corn.chain.kitchen`, {
      name: "Chain Kitchen - Corn",
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
    http(`https://rpc.${cornTestnet.id}.corn.chain.kitchen`, {
      name: "Chain Kitchen - Corn Testnet",
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
    http(`https://evm-rpc.${sei.id}.sei.chain.kitchen`, {
      name: "Chain Kitchen - Sei",
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
    http(`https://evm-rpc.${seiTestnet.id}.sei.chain.kitchen`, {
      name: "Chain Kitchen - Sei Testnet",
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
    http(`https://rpc.${bsc.id}.bsc.chain.kitchen`, {
      name: "Chain Kitchen - BNB Chain",
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
    http(`https://rpc.${bscTestnet.id}.bsc.chain.kitchen`, {
      name: "Chain Kitchen - BNB Chain Testnet",
    }),
    http(bscTestnet.rpcUrls.default.http.at(0), { name: "default BSC Testnet RPC" }),
  ]),
}

function createWagmiConfigInstance() {
  const edition: Edition = uiStore.edition
  return createConfig({
    chains: VIEM_CHAINS,
    cacheTime: 4_000,
    pollingInterval: 4_000,
    syncConnectedChain: true,
    batch: { multicall: true },
    multiInjectedProviderDiscovery: true,
    transports,
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
  if (Option.isNone(wagmiConfig)) {
    wagmiConfig = Option.some(createWagmiConfigInstance())
  }
  return Option.match(wagmiConfig, {
    onSome: (config) => config,
    onNone: () => {
      throw new Error("no wagmi config found")
    },
  })
}

export type ConfiguredChainId = (typeof VIEM_CHAINS)[number]["id"]
