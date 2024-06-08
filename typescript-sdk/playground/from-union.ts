#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { UnionClient } from "#mod.ts"

/* `bun scripts/from-osmosis.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const unionClient = await UnionClient.connectWithSecret({
  secretType: "key",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" },
  rpcUrl: "https://rpc.testnet.bonlulu.uno"
})

// const unoFromUnionToOsmosis = await unionClient.ibcMessageTransfers([
//   {
//     sourcePort: "transfer",
//     sourceChannel: "channel-7775",
//     token: { denom: "uosmo", amount: "100" },
//     sender: "osmo14qemq0vw6y3gc3u3e0aty2e764u4gs5l32ydm0",
//     receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
//     memo: "sending wrapped OSMO from Osmosis to Union",
//     timeoutHeight: { revisionHeight: 888888888n, revisionNumber: 8n }
//   }
// ])

// console.log(unoFromUnionToOsmosis.transactionHash)

const unoFromOsmosisToUnion = await unionClient.ibcMessageTransfers([
  {
    sourcePort: "transfer",
    sourceChannel: "channel-7775",
    token: { denom: "uosmo", amount: "100" },
    sender: "osmo14qemq0vw6y3gc3u3e0aty2e764u4gs5l32ydm0",
    receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    memo: "sending wrapped OSMO from Osmosis to Union",
    timeoutHeight: { revisionHeight: 888888888n, revisionNumber: 8n }
  }
])

console.log(unoFromOsmosisToUnion.transactionHash)
