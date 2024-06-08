import type { PageLoad } from "./$types.ts"
import { assets, type Asset } from "$/lib/constants/assets.ts"

interface Chain {
  id: string
  name: string
  ecosystem: "evm" | "cosmos"
  icon: string
  live: boolean
}

export const load = (_context => {
  return {
    // chains: [
    // {
    //   name: "Union Testnet",
    //   id: "union-testnet-8",
    //   ecosystem: "cosmos",
    //   icon: "/images/icons/union.svg",
    //   live: true,
    //   assets: [
    //     {
    //       client: "07-tendermint-20",
    //       connection: "connection-7",
    //       channel: "channel-6",
    //       port: "wasm.union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    //       contractAddress: "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7",
    //       destination: "union-testnet-8",
    //       symbol: "OSMO"
    //     }
    //   ]
    // },
    // {
    //   name: "Sepolia",
    //   ecosystem: "evm",
    //   icon: "/images/icons/ethereum.svg",
    //   id: "11155111",
    //   live: true
    // },
    // {
    //   name: "Osmosis Testnet",
    //   ecosystem: "cosmos",
    //   icon: "/images/icons/osmosis.svg",
    //   id: "osmo-test-5",
    //   live: false,
    //   assets: [
    //     {
    //       port: "transfer",
    //       channel: "channel-7775",
    //       client: "08-wasm-3551",
    //       connection: "connection-3021",
    //       destination: "union-testnet-8",
    //       symbol: "OSMO"
    //     }
    //   ]
    // },
    // {
    //   name: "Stargaze Testnet",
    //   ecosystem: "cosmos",
    //   icon: "/images/icons/stargaze.svg",
    //   id: "stargaze-1",
    //   live: false
    // },
    // {
    //   name: "Scroll Testnet",
    //   ecosystem: "cosmos",
    //   icon: "/images/icons/scroll.svg",
    //   id: "534352",
    //   live: false
    // }
    // ] as Array<Chain>,
    assets: assets as Array<Asset>
  }
}) satisfies PageLoad<{ assets: Array<Asset> }>
