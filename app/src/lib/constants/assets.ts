export type ChainId = "union-testnet-8" | "11155111" | "osmo-test-5" | "stargaze-1" | "534352"

type Chains =
  | {
      sourceChainId: "union-testnet-8"
      destinationChainId: Exclude<ChainId, "union-testnet-8">
    }
  | {
      sourceChainId: Exclude<ChainId, "union-testnet-8">
      destinationChainId: "union-testnet-8"
    }
export type ChainAssets = {
  port: string
  channel: string
  client: string
  connection: string
  contractAddress?: string
  assets: Array<{
    symbol: string
    denom: string
    displayName: string
    // TODO:
    decimals?: number
  }>
} & Chains

export const assets_: Array<ChainAssets> = [
  {
    sourceChainId: "union-testnet-8",
    destinationChainId: "osmo-test-5",
    channel: "channel-6",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    connection: "connection-7",
    client: "07-tendermint-20",
    assets: [
      {
        symbol: "UNO",
        denom: "muno",
        displayName: "UNO"
      },
      {
        symbol: "OSMO",
        denom:
          "factory/union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7/0xc5775fca1b3285dc8b749d58b227527211c108b8d3",
        displayName: "unionOSMO"
      }
    ]
  },
  {
    sourceChainId: "11155111",
    destinationChainId: "union-testnet-8",
    channel: "channel-0",
    contractAddress: "0x3d0EB16AD2619666dbde1921282cd885b58eEefE",
    port: "0x3d0eb16ad2619666dbde1921282cd885b58eeefe",
    connection: "connection-1",
    client: "cometbls-0",
    assets: [
      {
        symbol: "UNO",
        denom: "muno",
        displayName: "ethUNO"
      }
    ]
  },
  {
    sourceChainId: "union-testnet-8",
    destinationChainId: "11155111",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    channel: "channel-0",
    port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    client: "08-wasm-0",
    connection: "connection-1",
    assets: [
      {
        symbol: "UNO",
        denom: "muno",
        displayName: "ethUNO"
      }
    ]
  },
  {
    sourceChainId: "osmo-test-5",
    destinationChainId: "union-testnet-8",
    channel: "channel-7775",
    port: "transfer",
    client: "08-wasm-3551",
    connection: "connection-3021",
    assets: [
      {
        symbol: "OSMO",
        denom: "mosmo",
        displayName: "OSMO"
      },
      // TODO:
      {
        symbol: "UNO",
        denom: "muno",
        displayName: "osmosisUNO"
      }
    ]
  }
]

export const assets = [
  // UNO from Union to Osmosis - validated
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
    kind: "cosmwasm",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    display: "UNO (native to Osmosis)",
    explorerLink: "https://testnet.bonlulu.uno/union/tx/",
    id: "union-osmosis-uno"
  },
  // UNO from Union to Sepolia
  {
    source: {
      chain: "union-testnet-8",
      channel: "channel-0",
      client: "08-wasm-0",
      connection: "connection-1",
      port: "",
      explorerLink: "https://api.bonlulu.uno/union/tx/"
    },
    destination: {
      chain: "11155111",
      channel: "channel-0",
      port: "",
      client: "",
      connection: "",
      explorerLink: "https://sepolia.etherscan.io/tx/"
    },
    denom: "muno",
    symbol: "UNO",
    kind: "cosmwasm",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    display: "UNO (native to Sepolia)",
    id: "union-sepolia-uno"
  },
  // UNO from Osmosis to Union - validated
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
    denom: "muno",
    symbol: "UNO",
    kind: "ibc",
    display: "UNO (Osmosis to native)",
    contractAddress: null,
    explorerLink: "https://www.mintscan.io/osmosis-testnet/tx/",
    id: "osmosis-union-uno"
  },
  // OSMO from Osmosis to Union - validated
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
    // contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    contractAddress: null,
    kind: "cosmwasm",
    display: "OSMO (native to Union)",
    explorerLink: "https://www.mintscan.io/osmosis-testnet/tx/",
    id: "osmosis-union-osmo"
  },
  // OSMO from Union to Osmosis - validated
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
    kind: "ibc",
    contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    display: "UNO (Union to native)",
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
  kind: "ibc" | "cosmwasm" | "evm"
  contractAddress: string | null
  symbol: string
  denom: string
  display: string
  explorerLink?: string
  /**
   * id: source.chain + destination.chain + symbol
   */
  id: string
}
