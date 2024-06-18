#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { sepolia } from "viem/chains"
import { cosmosHttp } from "#transport.ts"
import { createUnionClient } from "#mod.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { consola } from "scripts/logger"

/* `bun playground/union-to-union.ts --private-key "..."` */

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

try {
  const client = createUnionClient({
    evm: {
      chain: sepolia,
      account: evmAccount,
      transport: http("https://rpc2.sepolia.org")
    },
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.025", denom: "muno" },
      transport: cosmosHttp("https://rpc.testnet.bonlulu.uno")
    }
  })

  const gasEstimationResponse = await client.simulateTransaction({
    amount: 1n,
    network: "cosmos",
    denomAddress: "muno",
    path: ["union-testnet-8", "union-testnet-8"],
    recipient: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
  })
  consola.info(`Gas cost: ${gasEstimationResponse.data}`)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  // @ts-expect-error
  const transfer = await client.transferAsset({
    amount: 1n,
    network: "cosmos",
    denomAddress: "muno",
    path: ["union-testnet-8", "union-testnet-8"],
    recipient: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
