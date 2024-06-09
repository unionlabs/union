#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { UnionClient } from "#mod.ts"

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

const contractAddress = "union1eumfw2ppz8cwl8xdh3upttzp5rdyms48kqhm30f8g9u4zwj0pprqg2vmu3"
const unoFromUnionToOsmosis = await unionClient.transferAssets({
  kind: "cosmwasm",
  instructions: [
    {
      contractAddress,
      msg: {
        transfer: {
          channel: "channel-27",
          receiver: "osmo14qemq0vw6y3gc3u3e0aty2e764u4gs5l32ydm0",
          memo: "sending UNO from Union to Osmosis"
        }
      },
      funds: [
        {
          amount: "1",
          denom: `muno`
          // denom: `factory/${contractAddress}/0xc5775fca1b3285dc8b749d58b227527211c108b8d3`
        }
      ]
    }
  ]
})

console.log(unoFromUnionToOsmosis.transactionHash)
