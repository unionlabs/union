#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { createUnionClient } from "#mod.ts"
import { raise } from "#utilities/index.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/union-to-union.ts --private-key "..."` --estimate-gas */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) raise("Private key not found")
const ONLY_ESTIMATE_GAS = values["estimate-gas"] ?? false

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

try {
  const client = createUnionClient({
    account: cosmosAccount,
    chainId: "union-testnet-8",
    gasPrice: { amount: "0.0025", denom: "muno" },
    transport: http("https://rpc.testnet-8.union.build")
  })

  const transfer = await client.transferAsset({
    amount: 1n,
    autoApprove: true,
    denomAddress: "muno",
    destinationChainId: "stride-internal-1",
    receiver: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3"
  })

  consola.info(transfer)
  process.exit(0)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
