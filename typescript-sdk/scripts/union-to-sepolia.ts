#!/usr/bin/env bun
import "#/patch.ts"
import { parseArgs } from "node:util"
import { UnionClient } from "#/mod.ts"
import { consola, timestamp } from "./logger.ts"
import type { ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"

/* `bun scripts/to-sepolia.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "tx-count": { type: "string", default: "1" }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")
const TX_COUNT = Number(values["tx-count"])

consola.box(`Sending ${TX_COUNT} transactions from Union to Sepolia`)

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://rpc.testnet.bonlulu.uno",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" }
})

const contractAddress = "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7"

const unionToSepoliaTransactions: Array<ExecuteInstruction> = Array.from(
  { length: TX_COUNT },
  (_, index) => ({
    contractAddress,
    msg: {
      transfer: {
        channel: "channel-0",
        receiver: "0xD0081080Ae8493cf7340458Eaf4412030df5FEEb".slice(2),
        memo: `${index} - ${timestamp()} Sending UNO from Union to Sepolia`
      }
    },
    funds: [{ amount: (index + 1).toString(), denom: `muno` }]
  })
)

const transactionResults = await Array.fromAsync(
  unionToSepoliaTransactions,
  async transaction =>
    unionClient.transferAssets({ kind: "cosmwasm", instructions: [transaction] }),
  { concurrency: 1 }
)

consola.info(
  JSON.stringify(
    transactionResults.map(item => item.transactionHash),
    undefined,
    2
  )
)
