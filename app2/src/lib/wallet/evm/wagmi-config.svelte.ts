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
import { flow, pipe } from "effect/Function"
import type { Chain, Chain as ViemChain, FallbackTransport, Transport } from "viem"

let wagmiConfig: Option.Option<ReturnType<typeof createWagmiConfigInstance>> = Option.none()

type Transports = {
  [Item in (typeof VIEM_CHAINS)[number] as Item["id"]]: FallbackTransport<Transport[]>
}

export const ownedFallbacks: Transports = {
  [mainnet.id]: fallback([
    http(mainnet.rpcUrls.default.http.at(0), {
      name: "default Mainnet RPC",
    }),
    http(`https://rpc.1.ethereum.chain.kitchen`, {
      name: "Chain Kitchen - Ethereum",
    }),
  ]),
  [sepolia.id]: fallback([
    http(sepolia.rpcUrls.default.http.at(0), {
      name: "default Sepolia RPC",
    }),
    http(`https://rpc.11155111.ethereum.chain.kitchen`, {
      name: "Chain Kitchen - Sepolia",
    }),
  ]),
  [holesky.id]: fallback([
    http(holesky.rpcUrls.default.http.at(0), {
      name: "default Holesky RPC",
    }),
    http(`https://rpc.17000.ethereum.chain.kitchen`, {
      name: "Chain Kitchen - Holesky",
    }),
  ]),
  [berachainTestnetbArtio.id]: fallback([
    http(berachainTestnetbArtio.rpcUrls.default.http.at(0), {
      name: "default Berachain RPC",
    }),
  ]),
  [arbitrumSepolia.id]: fallback([
    http(arbitrumSepolia.rpcUrls.default.http.at(0), {
      name: "default Arbitrum Sepolia RPC",
    }),
  ]),
  [scrollSepolia.id]: fallback([
    http(scrollSepolia.rpcUrls.default.http.at(0), {
      name: "default Scroll Sepolia RPC",
    }),
  ]),
  [bobSepolia.id]: fallback([
    http(bobSepolia.rpcUrls.default.http.at(0), {
      name: "default Bob Sepolia RPC",
    }),
    http(`https://rpc.808813.bob.chain.kitchen`, {
      name: "Chain Kitchen - BOB Sepolia",
    }),
  ]),
  [bob.id]: fallback([
    http(bob.rpcUrls.default.http.at(0), { name: "default Bob RPC" }),
    http(`https://rpc.60808.bob.chain.kitchen`, {
      name: "Chain Kitchen - BOB",
    }),
  ]),
  [corn.id]: fallback([
    http(corn.rpcUrls.default.http.at(0), { name: "default Corn RPC" }),
    http(`https://rpc.21000000.corn.chain.kitchen`, {
      name: "Chain Kitchen - Corn",
    }),
  ]),
  [cornTestnet.id]: fallback([
    http(cornTestnet.rpcUrls.default.http.at(0), {
      name: "default Corn Testnet RPC",
    }),
    http(`https://rpc.21000001.corn.chain.kitchen`, {
      name: "Chain Kitchen - Corn Testnet",
    }),
  ]),
  [sei.id]: fallback([
    http(sei.rpcUrls.default.http.at(0), { name: "default Sei RPC" }),
    http(`https://evm-rpc.1329.sei.chain.kitchen`, {
      name: "Chain Kitchen - Sei",
    }),
  ]),
  [seiTestnet.id]: fallback([
    http(seiTestnet.rpcUrls.default.http.at(0), { name: "default Sei Testnet RPC" }),
    http(`https://evm-rpc.1328.sei.chain.kitchen`, {
      name: "Chain Kitchen - Sei Testnet",
    }),
  ]),
  [bsc.id]: fallback([
    http(bsc.rpcUrls.default.http.at(0), { name: "default BSC RPC" }),
    http(`https://rpc.56.bsc.chain.kitchen`, {
      name: "Chain Kitchen - BNB Chain",
    }),
  ]),
  [bscTestnet.id]: fallback([
    http(bscTestnet.rpcUrls.default.http.at(0), { name: "default BSC Testnet RPC" }),
    http(`https://rpc.97.bsc.chain.kitchen`, {
      name: "Chain Kitchen - BNB Chain Testnet",
    }),
  ]),
}

export const fallbackTransport = flow(
  (chain: Chain) => ownedFallbacks[chain.id as (typeof VIEM_CHAINS)[number]["id"]],
)

const transports: Transports = {
  [mainnet.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-mainnet",
      name: "unstable_connector-injected-mainnet",
    }),
    ownedFallbacks[mainnet.id],
  ]),
  [sepolia.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-sepolia",
      name: "unstable_connector-injected-sepolia",
    }),
    ownedFallbacks[sepolia.id],
  ]),
  [holesky.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-holesky",
      name: "unstable_connector-injected-holesky",
    }),
    ownedFallbacks[holesky.id],
  ]),
  [berachainTestnetbArtio.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-berachain",
      name: "unstable_connector-injected-berachain",
    }),
    ownedFallbacks[berachainTestnetbArtio.id],
  ]),
  [arbitrumSepolia.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-arbitrum-sepolia",
      name: "unstable_connector-injected-arbitrum-sepolia",
    }),
    ownedFallbacks[arbitrumSepolia.id],
  ]),
  [scrollSepolia.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-scroll-sepolia",
      name: "unstable_connector-injected-scroll-sepolia",
    }),
    ownedFallbacks[scrollSepolia.id],
  ]),
  [bobSepolia.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-bob-sepolia",
      name: "unstable_connector-injected-bob-sepolia",
    }),
    ownedFallbacks[bobSepolia.id],
  ]),
  [bob.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-bob",
      name: "unstable_connector-injected-bob",
    }),
    ownedFallbacks[bob.id],
  ]),
  [corn.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-corn",
      name: "unstable_connector-injected-corn",
    }),
    ownedFallbacks[corn.id],
  ]),
  [cornTestnet.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-corn-testnet",
      name: "unstable_connector-injected-corn-testnet",
    }),
    ownedFallbacks[cornTestnet.id],
  ]),
  [sei.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-sei",
      name: "unstable_connector-injected-sei",
    }),
    ownedFallbacks[sei.id],
  ]),
  [seiTestnet.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-sei-testnet",
      name: "unstable_connector-injected-sei-testnet",
    }),
    ownedFallbacks[seiTestnet.id],
  ]),
  [bsc.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-bsc",
      name: "unstable_connector-injected-bsc",
    }),
    ownedFallbacks[bsc.id],
  ]),
  [bscTestnet.id]: fallback([
    unstable_connector(injected, {
      retryCount: 3,
      retryDelay: 100,
      key: "unstable_connector-injected-bsc-testnet",
      name: "unstable_connector-injected-bsc-testnet",
    }),
    ownedFallbacks[bscTestnet.id],
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
