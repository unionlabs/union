export type ChainId = "union-testnet-8" | "11155111" | "osmo-test-5" | "stargaze-1" | "534352"

export const assets = [
  {
    source: {
      port: "transfer",
      channel: "channel-7775",
      client: "08-wasm-3551",
      connection: "connection-3021",
      chain: "osmo-test-5",
      explorerLink: "https://www.mintscan.io/osmosis-testnet/tx/"
    },
    destination: {
      port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
      channel: "channel-6",
      client: "07-tendermint-20",
      connection: "connection-7",
      chain: "union-testnet-8",
      explorerLink: "https://testnet.bonlulu.uno/union/tx/"
    },
    symbol: "OSMO",
    denom: "uosmo",
    display: "unionOSMO",
    id: "osmosis-union-osmo"
  },
  {
    source: {
      port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
      channel: "channel-6",
      client: "07-tendermint-20",
      connection: "connection-7",
      chain: "union-testnet-8",
      explorerLink: "https://testnet.bonlulu.uno/union/tx/"
    },
    destination: {
      port: "transfer",
      channel: "channel-7775",
      client: "08-wasm-3551",
      connection: "connection-3021",
      chain: "osmo-test-5",
      explorerLink: "https://www.mintscan.io/osmosis-testnet/tx/"
    },
    denom:
      "factory/union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7/0xc5775fca1b3285dc8b749d58b227527211c108b8d3",
    symbol: "OSMO",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    display: "OSMO",
    explorerLink: "https://testnet.bonlulu.uno/union/tx/",
    id: "union-osmosis-osmo"
  },
  {
    source: {
      port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
      channel: "channel-6",
      client: "07-tendermint-20",
      connection: "connection-7",
      chain: "union-testnet-8",
      explorerLink: "https://testnet.bonlulu.uno/union/tx/"
    },
    destination: {
      port: "transfer",
      channel: "channel-7775",
      client: "08-wasm-3551",
      connection: "connection-3021",
      chain: "osmo-test-5",
      explorerLink: "https://www.mintscan.io/osmosis-testnet/tx/"
    },
    denom: "muno",
    symbol: "UNO",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    display: "osmosisUNO",
    explorerLink: "https://testnet.bonlulu.uno/union/tx/",
    id: "union-osmosis-uno"
  },
  {
    source: {
      port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
      channel: "channel-0",
      client: "08-wasm-0",
      connection: "connection-1",
      chain: "union-testnet-8",
      explorerLink: "https://sepolia.etherscan.io/tx"
    },
    destination: {
      port: "transfer",
      channel: "channel-7775",
      client: "08-wasm-3551",
      connection: "connection-3021",
      chain: "osmo-test-5",
      explorerLink: "https://www.mintscan.io/osmosis-testnet/tx/"
    },
    denom: "muno",
    symbol: "UNO",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    display: "ethUNO",
    explorerLink: "https://sepolia.etherscan.io/tx",
    id: "union-osmosis-eth-uno"
  }
] satisfies Array<Asset>

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
