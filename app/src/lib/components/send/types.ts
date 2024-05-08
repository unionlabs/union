export interface Asset {
  destination: "union-testnet-8" | "11155111" | "osmo-test-5" | "stargaze-1" | "534352"
  port: string
  client: string
  channel: string
  connection: string
  contractAddress?: string
  symbol: string
  denom: string
  display: string
  explorerLink?: string
}
