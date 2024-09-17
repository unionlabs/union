#!/usr/bin/env bun
import "#patch.ts"
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/stride-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")
const ONLY_ESTIMATE_GAS = values["estimate-gas"] ?? false

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "stride"
)
const [account] = await cosmosAccount.getAccounts()
console.info(account?.address)

try {
  const client = createUnionClient({
    account: cosmosAccount,
    chainId: "stride-internal-1",
    gasPrice: { amount: "0.0025", denom: "ustrd" },
    transport: http("https://stride-testnet-rpc.polkachu.com")
  })

  const transactionPayload = {
    amount: 1n,
    autoApprove: true,
    denomAddress: "ustrd",
    destinationChainId: "union-testnet-8",
    receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
  } satisfies TransferAssetsParameters<"stride-internal-1">

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  consola.box("Stride to Union gas cost:", gasEstimationResponse)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (gasEstimationResponse.isErr()) {
    consola.info("Transaction simulation failed", gasEstimationResponse.error)
    process.exit(1)
  }

  consola.success("Stride to Union gas cost:", gasEstimationResponse.value)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (gasEstimationResponse.isErr()) {
    console.info("Transaction simulation failed", gasEstimationResponse.error)
    process.exit(1)
  }

  const transfer = await client.transferAsset(transactionPayload)

  if (transfer.isErr()) {
    console.error(transfer.error)
    process.exit(1)
  }

  consola.info(transfer.value)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
