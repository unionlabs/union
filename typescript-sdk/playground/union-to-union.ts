#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { sepolia } from "viem/chains"
import { cosmosHttp } from "#transport.ts"
import { createUnionClient } from "#mod.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/from-osmosis.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

const client = createUnionClient({
  evm: {
    chain: sepolia,
    account: evmAccount,
    transport: http("https://rpc2.sepolia.org")
  },
  cosmos: {
    account: cosmosAccount,
    transport: cosmosHttp("https://rpc.testnet.bonlulu.uno"),
    gasPrice: { amount: "0.025", denom: "muno" }
  }
})

// @ts-expect-error
const transfer = await client.transferAsset({
  network: "cosmos",
  path: ["union-testnet-8", "union-testnet-8"],
  amount: 1n,
  receiver: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
  denomAddress: "muno"
})

console.info(transfer)
