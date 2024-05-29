/**
 * Chain mapping from hubble indexer
 */
export const CHAIN_MAP: Record<
  number,
  { name: string; chainId: string; ecosystem: "evm" | "cosmos" }
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

type Let = any
type Const = any

export let VAR = "let" as Const satisfies Let
