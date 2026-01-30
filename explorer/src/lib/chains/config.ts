export interface ChainAsset {
  symbol: string
  base: string
  exponent: number
  coingecko_id?: string
  logo?: string
}

export interface SocialLink {
  type: "twitter" | "discord" | "github" | "telegram" | "website"
  url: string
}

export interface ChainConfig {
  chain_name: string
  chain_id: string // Native chain ID (e.g., "union-1")
  universal_chain_id: string // UCS04 universal chain ID (e.g., "union.union-1")
  pretty_name: string
  api: string[]
  rpc: string[]
  addr_prefix: string
  coin_type: string
  theme_color: string
  logo?: string
  assets: ChainAsset[]
  features: string[]
  socials?: SocialLink[]
}

// Union mainnet config
export const UNION_MAINNET: ChainConfig = {
  chain_name: "union",
  chain_id: "union-1",
  universal_chain_id: "union.union-1",
  pretty_name: "Union",
  api: [
    "https://union-api.polkachu.com",
    "https://rest.union.build",
    "https://union-rest.publicnode.com",
  ],
  rpc: [
    "https://rpc.union.build",
    "https://union-rpc.polkachu.com",
    "https://union-rpc.publicnode.com",
  ],
  addr_prefix: "union",
  coin_type: "118",
  theme_color: "#A0ECFD",
  logo: "/chains/union.svg",
  assets: [
    {
      symbol: "U",
      base: "au",
      exponent: 18,
      coingecko_id: "union-2",
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
  socials: [
    { type: "twitter", url: "https://x.com/union_build" },
    { type: "discord", url: "https://discord.gg/union-build" },
    { type: "github", url: "https://github.com/unionlabs" },
    { type: "website", url: "https://union.build" },
  ],
}

// Union testnet config
export const UNION_TESTNET: ChainConfig = {
  chain_name: "union-testnet",
  chain_id: "union-testnet-10",
  universal_chain_id: "union.union-testnet-10",
  pretty_name: "Union Testnet",
  api: ["https://rest.testnet-10.union.build"],
  rpc: ["https://rpc.testnet-10.union.build"],
  addr_prefix: "union",
  coin_type: "118",
  theme_color: "#A0ECFD",
  logo: "/chains/union.svg",
  assets: [
    {
      symbol: "U",
      base: "au",
      exponent: 18,
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
}

// Babylon mainnet
export const BABYLON_MAINNET: ChainConfig = {
  chain_name: "babylon",
  chain_id: "bbn-1",
  universal_chain_id: "babylon.bbn-1",
  pretty_name: "Babylon",
  api: [
    "https://babylon-rest.publicnode.com",
    "https://babylon-api.polkachu.com",
    "https://babylon.nodes.guru/api",
    "https://babylon-mainnet-lcd.autostake.com:443",
  ],
  rpc: [
    "https://babylon-rpc.publicnode.com",
    "https://babylon-rpc.polkachu.com",
  ],
  addr_prefix: "bbn",
  coin_type: "118",
  theme_color: "#FF7C2A",
  assets: [
    {
      symbol: "BBN",
      base: "ubbn",
      exponent: 6,
      coingecko_id: "babylon",
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
  socials: [
    { type: "twitter", url: "https://x.com/babylon_chain" },
    { type: "website", url: "https://babylonchain.io" },
  ],
}

// Osmosis mainnet
export const OSMOSIS_MAINNET: ChainConfig = {
  chain_name: "osmosis",
  chain_id: "osmosis-1",
  universal_chain_id: "osmosis.osmosis-1",
  pretty_name: "Osmosis",
  api: [
    "https://osmosis-rest.publicnode.com",
    "https://lcd.osmosis.zone",
    "https://osmosis-api.polkachu.com",
  ],
  rpc: [
    "https://osmosis-rpc.publicnode.com",
    "https://rpc.osmosis.zone",
    "https://osmosis-rpc.polkachu.com",
  ],
  addr_prefix: "osmo",
  coin_type: "118",
  theme_color: "#5E12A0",
  assets: [
    {
      symbol: "OSMO",
      base: "uosmo",
      exponent: 6,
      coingecko_id: "osmosis",
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
  socials: [
    { type: "twitter", url: "https://x.com/osmaboratory" },
    { type: "website", url: "https://osmosis.zone" },
  ],
}

// Neutron mainnet
export const NEUTRON_MAINNET: ChainConfig = {
  chain_name: "neutron",
  chain_id: "neutron-1",
  universal_chain_id: "neutron.neutron-1",
  pretty_name: "Neutron",
  api: [
    "https://neutron-rest.publicnode.com",
    "https://rest-lb.neutron.org",
    "https://neutron-api.polkachu.com",
  ],
  rpc: [
    "https://neutron-rpc.publicnode.com",
    "https://rpc-lb.neutron.org",
    "https://neutron-rpc.polkachu.com",
  ],
  addr_prefix: "neutron",
  coin_type: "118",
  theme_color: "#000000",
  assets: [
    {
      symbol: "NTRN",
      base: "untrn",
      exponent: 6,
      coingecko_id: "neutron-3",
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
  socials: [
    { type: "twitter", url: "https://x.com/Neutron_org" },
    { type: "website", url: "https://neutron.org" },
  ],
}

// Stargaze mainnet
export const STARGAZE_MAINNET: ChainConfig = {
  chain_name: "stargaze",
  chain_id: "stargaze-1",
  universal_chain_id: "stargaze.stargaze-1",
  pretty_name: "Stargaze",
  api: [
    "https://stargaze-rest.publicnode.com",
    "https://stargaze-api.polkachu.com",
    "https://rest.stargaze-apis.com",
  ],
  rpc: [
    "https://stargaze-rpc.publicnode.com",
    "https://stargaze-rpc.polkachu.com",
    "https://rpc.stargaze-apis.com",
  ],
  addr_prefix: "stars",
  coin_type: "118",
  theme_color: "#DB2777",
  assets: [
    {
      symbol: "STARS",
      base: "ustars",
      exponent: 6,
      coingecko_id: "stargaze",
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
  socials: [
    { type: "twitter", url: "https://x.com/StargazeZone" },
    { type: "website", url: "https://stargaze.zone" },
  ],
}

// Xion mainnet
export const XION_MAINNET: ChainConfig = {
  chain_name: "xion",
  chain_id: "xion-mainnet-1",
  universal_chain_id: "xion.xion-mainnet-1",
  pretty_name: "Xion",
  api: [
    "https://xion-api.polkachu.com",
    "https://api.xion-mainnet-1.burnt.com",
    "https://xion-api.lavenderfive.com",
  ],
  rpc: [
    "https://xion-rpc.polkachu.com",
    "https://rpc.xion-mainnet-1.burnt.com",
    "https://xion-rpc.lavenderfive.com",
  ],
  addr_prefix: "xion",
  coin_type: "118",
  theme_color: "#FFFFFF",
  assets: [
    {
      symbol: "XION",
      base: "uxion",
      exponent: 6,
      coingecko_id: "xion-2",
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
  socials: [
    { type: "twitter", url: "https://x.com/burnt_xion" },
    { type: "website", url: "https://burnt.com" },
  ],
}

// dYdX mainnet
export const DYDX_MAINNET: ChainConfig = {
  chain_name: "dydx",
  chain_id: "dydx-mainnet-1",
  universal_chain_id: "dydx.dydx-mainnet-1",
  pretty_name: "dYdX",
  api: [
    "https://dydx-rest.publicnode.com",
    "https://dydx-dao-api.polkachu.com",
    "https://rest.lavenderfive.com:443/dydx",
    "https://dydx-mainnet-lcd.autostake.com:443",
  ],
  rpc: [
    "https://dydx-rpc.publicnode.com",
    "https://dydx-dao-rpc.polkachu.com",
  ],
  addr_prefix: "dydx",
  coin_type: "118",
  theme_color: "#6966FF",
  assets: [
    {
      symbol: "DYDX",
      base: "adydx",
      exponent: 18,
      coingecko_id: "dydx-chain",
    },
  ],
  features: ["blocks", "transactions", "validators", "governance", "accounts", "ibc", "params"],
  socials: [
    { type: "twitter", url: "https://x.com/dYdX" },
    { type: "website", url: "https://dydx.exchange" },
  ],
}

// Use universal_chain_id as the key for URL routing
export const CHAINS: Record<string, ChainConfig> = {
  "union.union-1": UNION_MAINNET,
  "union.union-testnet-10": UNION_TESTNET,
  "babylon.bbn-1": BABYLON_MAINNET,
  "osmosis.osmosis-1": OSMOSIS_MAINNET,
  "neutron.neutron-1": NEUTRON_MAINNET,
  "stargaze.stargaze-1": STARGAZE_MAINNET,
  "xion.xion-mainnet-1": XION_MAINNET,
  "dydx.dydx-mainnet-1": DYDX_MAINNET,
}

export const DEFAULT_CHAIN = "union.union-1"

export const getChain = (name: string): ChainConfig | undefined => CHAINS[name]

export const getRandomEndpoint = (endpoints: string[]): string => {
  const index = Math.floor(Math.random() * endpoints.length)
  return endpoints[index]
}
