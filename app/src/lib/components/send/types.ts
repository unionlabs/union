export type ChainId = "union-testnet-8" | "11155111" | "osmo-test-5" | "stargaze-1" | "534352"

export interface Asset {
  source: {
    port: string
    channel: string
    client: string
    connection: string
    chain: ChainId
    explorerLink?: string
  }
  destination: {
    port: string
    channel: string
    client: string
    connection: string
    chain: ChainId
    explorerLink?: string
  }
  contractAddress?: string
  symbol: string
  denom: string
  display: string
  explorerLink?: string
  /**
   * id: source.chain + destination.chain + symbol
   */
  id: string
}
