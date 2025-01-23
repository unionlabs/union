#!/usr/bin/env bun
import { fallback, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { holesky, sepolia } from "viem/chains"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/sepolia-to-holesky.ts --private-key "..."` */

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

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789"
const RECEIVER = "0x153919669Edc8A5D0c8D1E4507c9CE60435A1177"

try {
  const client = createUnionClient({
    chainId: "11155111",
    account: evmAccount,
    transport: fallback([
      http("https://rpc.11155111.sepolia.chain.kitchen"),
      http(sepolia?.rpcUrls.default.http.at(0))
    ])
  })

  const transactionPayload = {
    amount: 420n,
    destinationChainId: `${holesky.id}`,
    receiver: RECEIVER,
    denomAddress: LINK_CONTRACT_ADDRESS,
    autoApprove: true
  } satisfies TransferAssetsParameters<"11155111">

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  if (gasEstimationResponse.isErr()) {
    consola.error(gasEstimationResponse.error)
    process.exit(1)
  }

  consola.success("Sepolia to Holesky gas cost:", gasEstimationResponse.value)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (gasEstimationResponse.isErr()) {
    console.info("Transaction simulation failed", gasEstimationResponse.error)
    process.exit(1)
  }

  const transfer = await client.transferAssetLegacy(transactionPayload)

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
