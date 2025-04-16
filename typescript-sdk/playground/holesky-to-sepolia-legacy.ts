#!/usr/bin/env bun
import { fallback, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "../src/utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { holesky, sepolia } from "viem/chains"
import { createUnionClient, type TransferAssetsParametersLegacy } from "../src/mod.ts"

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

const LINK_CONTRACT_ADDRESS = "0x685cE6742351ae9b618F383883D6d1e0c5A31B4B"
const RECEIVER = "0x153919669Edc8A5D0c8D1E4507c9CE60435A1177"

try {
  const client = createUnionClient({
    chainId: "17000",
    account: evmAccount,
    transport: fallback([
      http("https://rpc.holesky.sepolia.chain.kitchen"),
      http(holesky?.rpcUrls.default.http.at(0))
    ])
  })

  const transactionPayload = {
    amount: 421n,
    destinationChainId: `${sepolia.id}`,
    receiver: RECEIVER,
    denomAddress: LINK_CONTRACT_ADDRESS,
    autoApprove: true
  } satisfies TransferAssetsParametersLegacy<"17000">

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
