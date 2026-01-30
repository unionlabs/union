import { Context, Layer } from "effect"

// Chain configuration
export interface ChainConfig {
  id: string
  name: string
  rpc: string[] // Tendermint RPC endpoints
  rest: string[] // Cosmos REST endpoints
}

export const CHAINS: ChainConfig[] = [
  {
    id: "union.union-1",
    name: "Union",
    rpc: ["https://rpc.union.build", "https://union-rpc.polkachu.com"],
    rest: ["https://rest.union.build", "https://union-api.polkachu.com"],
  },
  {
    id: "babylon.bbn-1",
    name: "Babylon",
    rpc: ["https://babylon-rpc.publicnode.com", "https://babylon-rpc.polkachu.com"],
    rest: ["https://babylon-rest.publicnode.com", "https://babylon-api.polkachu.com"],
  },
  {
    id: "osmosis.osmosis-1",
    name: "Osmosis",
    rpc: ["https://osmosis-rpc.publicnode.com", "https://rpc.osmosis.zone"],
    rest: ["https://osmosis-rest.publicnode.com", "https://lcd.osmosis.zone"],
  },
  {
    id: "neutron.neutron-1",
    name: "Neutron",
    rpc: ["https://neutron-rpc.publicnode.com", "https://rpc-lb.neutron.org"],
    rest: ["https://neutron-rest.publicnode.com", "https://rest-lb.neutron.org"],
  },
  {
    id: "stargaze.stargaze-1",
    name: "Stargaze",
    rpc: ["https://stargaze-rpc.publicnode.com", "https://stargaze-rpc.polkachu.com"],
    rest: ["https://stargaze-rest.publicnode.com", "https://stargaze-api.polkachu.com"],
  },
  {
    id: "xion.xion-mainnet-1",
    name: "Xion",
    rpc: ["https://xion-rpc.polkachu.com", "https://rpc.xion-mainnet-1.burnt.com"],
    rest: ["https://xion-api.polkachu.com", "https://api.xion-mainnet-1.burnt.com"],
  },
  {
    id: "dydx.dydx-mainnet-1",
    name: "dYdX",
    rpc: ["https://dydx-rpc.publicnode.com", "https://dydx-dao-rpc.polkachu.com"],
    rest: ["https://dydx-rest.publicnode.com", "https://dydx-dao-api.polkachu.com"],
  },
]

// Indexer settings
export interface IndexerConfig {
  port: number
  pollInterval: number
  blocksToKeep: number
  backfillBatchSize: number
  dbPath: string
}

export class IndexerConfigService extends Context.Tag("IndexerConfig")<
  IndexerConfigService,
  IndexerConfig
>() {}

export const IndexerConfigLive = Layer.succeed(IndexerConfigService, {
  port: 3002,
  pollInterval: 5000,
  blocksToKeep: 100_000, // 100k blocks per chain
  backfillBatchSize: 20,
  dbPath: "indexer.sqlite",
})
