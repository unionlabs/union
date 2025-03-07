export const cosmosChainId = [
  "elgafar-1",
  "osmo-test-5",
  "union-testnet-9",
  "stride-internal-1",
  "bbn-test-5",
  "union-testnet-8"
] as const

export const cosmosRpcs: Record<CosmosChainId, string> = {
  "elgafar-1": "https://rpc.elgafar-1.stargaze.chain.kitchen",
  "osmo-test-5": "https://rpc.osmo-test-5.osmosis.chain.kitchen",
  "union-testnet-9": "https://rpc.testnet-9.union.build",
  "union-testnet-8": "https://rpc.union-testnet-8.union.chain.kitchen",
  "stride-internal-1": "https://rpc.stride-internal-1.stride.chain.kitchen",
  "bbn-test-5": "https://rpc.bbn-test-5.babylon.chain.kitchen"
}

export type CosmosChainId = `${(typeof cosmosChainId)[number]}`