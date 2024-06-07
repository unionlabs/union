#!/usr/bin/env bun
import "#patch.ts"
import { parseArgs } from "node:util"
import { UnionClient } from "#mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { consola, timestamp } from "../scripts/logger.ts"
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

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

consola.box(`Sending ${TX_COUNT} transactions from Union to Sepolia`)

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://rpc.testnet.bonlulu.uno",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" }
})

const { address } = await unionClient.getCosmosSdkAccount()

const contractAddress = "union1eumfw2ppz8cwl8xdh3upttzp5rdyms48kqhm30f8g9u4zwj0pprqg2vmu3"
const stamp = timestamp()
const unionToSepoliaTransactions: Array<ExecuteInstruction> = Array.from(
  { length: TX_COUNT },
  (_, index) => ({
    contractAddress,
    msg: {
      transfer: {
        channel: "channel-23",
        receiver: evmAccount.address.slice(2),
        memo: `${index} - ${stamp} Sending UNO from Union to ${evmAccount.address} on Sepolia`
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
console.info(stamp)
consola.info(
  JSON.stringify(
    transactionResults.map(item => item.transactionHash),
    undefined,
    2
  )
)
