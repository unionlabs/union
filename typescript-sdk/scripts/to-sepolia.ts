#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { UnionClient } from "#/mod.ts"

/* `bun scripts/to-sepolia.ts --private-key "..."` */

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
const osmoFromUnionToSepolia = await unionClient.transferAssets({
  kind: "cosmwasm",
  instructions: [
    {
      contractAddress,
      msg: {
        transfer: {
          channel: "channel-0",
          receiver: "0x50C9C35e0197e781e9aD7a3F6baDD8d01E45c377".slice(2),
          memo: "sending wrapped OSMO from Union to Sepolia"
        }
      },
      funds: [
        {
          amount: "30",
          denom: `factory/union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7/0xc5775fca1b3285dc8b749d58b227527211c108b8d3`
        }
      ]
    }
  ]
})

console.log(osmoFromUnionToSepolia.transactionHash)
