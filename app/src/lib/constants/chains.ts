export const chainIds = [
  "union-testnet-8",
  "11155111",
  "osmo-test-5",
  "534352",
  "elgafar-1",
  "mocha-4",
  "80085",
  "421614",
  "534351"
] as const
export type ChainId = (typeof chainIds)[number]

/**
 * Chain mapping from hubble indexer
 */
export const chainMap: Record<
  number,
  { name: string; chainId: ChainId; ecosystem: "evm" | "cosmos" }
> = {
  1: { name: "sepolia", chainId: "11155111", ecosystem: "evm" },
  5: { name: "stargaze", chainId: "elgafar-1", ecosystem: "cosmos" },
  4: { name: "union", chainId: "union-testnet-8", ecosystem: "cosmos" },
  6: { name: "osmosis", chainId: "osmo-test-5", ecosystem: "cosmos" },
  7: { name: "celestia", chainId: "mocha-4", ecosystem: "cosmos" },
  8: { name: "berachain", chainId: "80085", ecosystem: "evm" },
  9: { name: "scroll", chainId: "534351", ecosystem: "evm" },
  10: { name: "arbitrum", chainId: "421614", ecosystem: "evm" }
}
