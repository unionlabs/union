import type { PageLoad } from "./$types.ts"
import type { Asset } from "$/lib/components/send/types.ts"

interface Chain {
  id: string
  name: string
  ecosystem: "evm" | "cosmos"
  icon: string
  live: boolean
}

export const load = (_context => {
  return {
    chains: [
      {
        name: "union",
        id: "union-testnet-8",
        ecosystem: "cosmos",
        icon: "/images/icons/union.svg",
        live: true,
        assets: [
          {
            client: "07-tendermint-20",
            connection: "connection-7",
            channel: "channel-6",
            port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
            contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
            destination: "union-testnet-8",
            symbol: "OSMO"
          }
        ]
      },
      {
        name: "sepolia",
        ecosystem: "evm",
        icon: "/images/icons/ethereum.svg",
        id: "11155111",
        live: true
      },
      {
        name: "osmosis",
        ecosystem: "cosmos",
        icon: "/images/icons/osmosis.svg",
        id: "osmo-test-5",
        live: true,
        assets: [
          {
            port: "transfer",
            channel: "channel-7775",
            client: "08-wasm-3551",
            connection: "connection-3021",
            destination: "union-testnet-8",
            symbol: "OSMO"
          }
        ]
      },
      {
        name: "stargaze",
        ecosystem: "cosmos",
        icon: "/images/icons/stargaze.svg",
        id: "stargaze-1",
        live: false
      },
      {
        name: "scroll",
        ecosystem: "cosmos",
        icon: "/images/icons/scroll.svg",
        id: "534352",
        live: false
      }
    ] as Array<Chain>,
    assets: [
      {
        destination: "union-testnet-8",
        channel: "channel-7775",
        port: "transfer",
        client: "07-tendermint-20",
        connection: "connection-7",
        denom: "uosmo",
        symbol: "OSMO",
        display: "unionOSMO",
        explorerLink: "https://www.mintscan.io/osmosis-testnet/tx/"
      },
      {
        destination: "osmo-test-5",
        channel: "channel-6",
        port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
        client: "07-tendermint-20",
        connection: "connection-7",
        denom:
          "factory/union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7/0xc5775fca1b3285dc8b749d58b227527211c108b8d3",
        symbol: "OSMO",
        contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
        display: "OSMO",
        explorerLink: "https://testnet.bonlulu.uno/union/tx/"
      },
      {
        destination: "osmo-test-5",
        channel: "channel-6",
        denom: "muno",
        symbol: "UNO",
        port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
        client: "07-tendermint-20",
        connection: "connection-7",
        contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
        display: "osmosisUNO",
        explorerLink: "https://testnet.bonlulu.uno/union/tx/"
      },
      {
        destination: "11155111",
        channel: "channel-0",
        port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
        client: "08-wasm-0",
        connection: "connection-1",
        denom: "muno",
        symbol: "UNO",
        contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
        display: "ethUNO",
        explorerLink: "https://sepolia.etherscan.io/tx"
      }
    ] as Array<Asset>
  }
}) satisfies PageLoad<{ chains: Array<Chain>; assets: Array<Asset> }>
