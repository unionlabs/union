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
const unoFromUnionToSepolia = await unionClient.transferAssets({
  kind: "cosmwasm",
  instructions: [
    {
      contractAddress,
      msg: {
        transfer: {
          channel: "channel-0",
          receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd".slice(2),
          memo: "sending UNO from Union to Sepolia"
        }
      },
      funds: [
        // denom: `factory/union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7/0xc5775fca1b3285dc8b749d58b227527211c108b8d3`
        { amount: "3000", denom: `muno` }
      ]
    }
  ]
})

console.log(unoFromUnionToSepolia.transactionHash)
