#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { UnionClient } from "#/mod.ts"

/* `bun scripts/to-osmosis.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://rpc.testnet.bonlulu.uno",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" }
})

const contractAddress = "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7"
const osmoFromUnionToOsmosis = await unionClient.cosmwasmMessageExecuteContract([
  {
    contractAddress,
    msg: {
      transfer: {
        channel: "channel-6",
        receiver: "osmo14qemq0vw6y3gc3u3e0aty2e764u4gs5l32ydm0",
        memo: "sending wrapped OSMO from Union to Osmosis"
      }
    },
    funds: [
      {
        amount: "2",
        denom: `factory/${contractAddress}/0xc5775fca1b3285dc8b749d58b227527211c108b8d3`
      }
    ]
  }
])

console.log(osmoFromUnionToOsmosis.transactionHash)
